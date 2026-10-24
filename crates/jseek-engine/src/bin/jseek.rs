//! `jseek` command line: index a folder and search it with the JSeek query language.
//!
//!   jseek search <query> [--root DIR | --index FILE] [--limit N] [--json]
//!   jseek index <DIR> <FILE>        scan DIR and save the index to FILE
//!   jseek info                      version and scanning backend

use jseek_engine::{backend, Engine};
use std::process::ExitCode;

const USAGE: &str = "usage:
  jseek search <query> [--root DIR | --index FILE] [--limit N] [--json]
  jseek index <DIR> <FILE>
  jseek info";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("jseek: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("info") => {
            println!("jseek {}  backend: {}", jseek_core::version(), backend());
            Ok(())
        }
        Some("index") => {
            let (dir, file) = match (args.get(1), args.get(2)) {
                (Some(d), Some(f)) => (d, f),
                _ => return Err(USAGE.into()),
            };
            let mut e = Engine::new();
            let n = e.scan(dir);
            if !e.save(file) {
                return Err(format!("could not write index {file}"));
            }
            println!("indexed {n} entries from {dir} -> {file}");
            Ok(())
        }
        Some("search") => search(&args[1..]),
        _ => Err(USAGE.into()),
    }
}

fn search(args: &[String]) -> Result<(), String> {
    let mut query: Option<&str> = None;
    let mut root: Option<&str> = None;
    let mut index: Option<&str> = None;
    let mut limit: u32 = 100;
    let mut json = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--root" => root = Some(it.next().ok_or("--root needs a value")?),
            "--index" => index = Some(it.next().ok_or("--index needs a value")?),
            "--limit" => {
                limit = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--limit needs a number")?
            }
            "--json" => json = true,
            q if query.is_none() => query = Some(q),
            other => return Err(format!("unexpected argument {other}\n{USAGE}")),
        }
    }
    let query = query.ok_or(USAGE)?;
    let mut e = Engine::new();
    match (root, index) {
        (_, Some(f)) => {
            if !e.load(f) {
                return Err(format!("could not load index {f}"));
            }
        }
        (Some(d), None) => {
            e.scan(d);
        }
        (None, None) => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| "no --root given and no home directory")?;
            e.scan(home);
        }
    }
    let hits = e.search(query, limit);
    if json {
        let v: Vec<serde_json::Value> = hits
            .iter()
            .map(|h| {
                serde_json::json!({
                    "path": h.full_path(), "name": h.name, "dir": h.dir,
                    "kind": h.kind, "size": h.size, "modified": h.modified,
                })
            })
            .collect();
        println!("{}", serde_json::Value::Array(v));
    } else {
        for h in &hits {
            println!("{}", h.full_path());
        }
    }
    Ok(())
}
