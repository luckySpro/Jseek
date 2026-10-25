//! Grep-style content search (as in 1.x): read candidate text files and report matching lines.

use serde::Serialize;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LineHit {
    pub line: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentHit {
    pub path: String,
    pub name: String,
    pub dir: String,
    pub size: u64,
    pub modified: i64,
    pub lines: Vec<LineHit>,
}

const MAX_LINES_PER_FILE: usize = 5;
const MAX_LINE_CHARS: usize = 220;

/// Matching lines in `text` (case-insensitive). Long lines are cut around the match.
pub fn match_lines(text: &str, needle_lower: &str) -> Vec<LineHit> {
    let mut out = Vec::new();
    if needle_lower.is_empty() {
        return out;
    }
    for (i, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        if let Some(pos) = lower.find(needle_lower) {
            out.push(LineHit {
                line: i as u32 + 1,
                text: clip(line, &lower, pos),
            });
            if out.len() >= MAX_LINES_PER_FILE {
                break;
            }
        }
    }
    out
}

fn clip(line: &str, lower: &str, pos: usize) -> String {
    let line = line.trim_end();
    if line.chars().count() <= MAX_LINE_CHARS {
        return line.trim_start().to_string();
    }
    // Map the byte position in `lower` to a char index (lowercasing can change byte lengths).
    let char_pos = lower[..pos].chars().count();
    let start = char_pos.saturating_sub(60);
    let s: String = line.chars().skip(start).take(MAX_LINE_CHARS).collect();
    if start > 0 {
        format!("…{}", s.trim_start())
    } else {
        s
    }
}

/// Read a file as text if it is not too big and does not look binary.
pub fn read_text(path: &str, max_bytes: u64) -> Option<String> {
    let f = std::fs::File::open(path).ok()?;
    if f.metadata().ok()?.len() > max_bytes {
        return None;
    }
    let mut buf = Vec::new();
    f.take(max_bytes).read_to_end(&mut buf).ok()?;
    if buf.iter().take(8192).any(|&b| b == 0) {
        return None;
    }
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// Search `files` (path, name, dir, size, modified) in parallel; stops after `limit` files hit.
pub fn search(
    files: Vec<(String, String, String, u64, i64)>,
    needle: &str,
    max_bytes: u64,
    limit: usize,
) -> Vec<ContentHit> {
    let needle = needle.to_lowercase();
    let next = AtomicUsize::new(0);
    let found = AtomicUsize::new(0);
    let hits = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                if found.load(Ordering::Relaxed) >= limit {
                    break;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((path, name, dir, size, modified)) = files.get(i) else {
                    break;
                };
                let Some(text) = read_text(path, max_bytes) else {
                    continue;
                };
                let lines = match_lines(&text, &needle);
                if !lines.is_empty() {
                    found.fetch_add(1, Ordering::Relaxed);
                    hits.lock().unwrap().push((
                        i,
                        ContentHit {
                            path: path.clone(),
                            name: name.clone(),
                            dir: dir.clone(),
                            size: *size,
                            modified: *modified,
                            lines,
                        },
                    ));
                }
            });
        }
    });
    let mut v = hits.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i); // keep the engine's ranking order
    v.into_iter().map(|(_, h)| h).take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_lines_case_insensitively() {
        let hits = match_lines("alpha\nBeta Gamma\nbeta", "beta");
        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[0],
            LineHit {
                line: 2,
                text: "Beta Gamma".into()
            }
        );
        assert_eq!(hits[1].line, 3);
    }

    #[test]
    fn clips_long_lines_around_match() {
        let line = format!("{}NEEDLE{}", "x".repeat(400), "y".repeat(400));
        let hits = match_lines(&line, "needle");
        assert!(hits[0].text.contains("NEEDLE"));
        assert!(hits[0].text.chars().count() <= MAX_LINE_CHARS + 1);
    }

    #[test]
    fn skips_binary_files() {
        let dir = std::env::temp_dir().join(format!("jseek_content_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("a.bin");
        std::fs::write(&bin, b"needle\0\0").unwrap();
        let txt = dir.join("a.txt");
        std::fs::write(&txt, "one\nthe needle here\n").unwrap();
        let f = |p: &std::path::Path| {
            (
                p.to_string_lossy().into_owned(),
                String::new(),
                String::new(),
                0,
                0,
            )
        };
        let hits = search(vec![f(&bin), f(&txt)], "Needle", 1 << 20, 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].lines[0].line, 2);
        let _ = std::fs::remove_dir_all(dir);
    }
}
