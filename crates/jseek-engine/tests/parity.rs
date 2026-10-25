//! M2 acceptance: searches through the Rust bridge give the same answers as the 1.9 engine
//! (same fixture and expectations as C++ Part 56, test_query_filters.h).

use jseek_engine::Engine;
use std::fs;
use std::path::PathBuf;

/// Each test gets its own folder: tests run in parallel in one process.
fn fixture(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("jseek_parity_{}_{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("subdir/deep")).unwrap();
    let write = |rel: &str, n: usize| fs::write(root.join(rel), vec![b'x'; n]).unwrap();
    write("hello.cpp", 1025);
    write("world.h", 512);
    write("readme.md", 2048);
    write("big.txt", 1024 * 1024);
    write("small.log", 10);
    write("subdir/nested.cpp", 100);
    write("subdir/deep/buried.h", 50);
    root
}

fn names(e: &Engine, q: &str) -> Vec<String> {
    e.search(q, 100).into_iter().map(|h| h.name).collect()
}

#[test]
fn query_language_matches_engine_1_9() {
    let root = fixture("query");
    let mut e = Engine::new();
    assert!(e.scan(&root) >= 9, "scan found the fixture");

    let has = |q: &str, n: &str| names(&e, q).iter().any(|x| x == n);

    assert!(has("ext:cpp", "hello.cpp"));
    assert!(has("ext:cpp", "nested.cpp"));
    assert!(!has("ext:cpp", "world.h"));
    assert!(has("ext:cpp;h", "buried.h"));
    assert!(!has("ext:cpp;h", "readme.md"));

    assert!(!has("size:>1kb", "small.log"));
    assert!(has("size:>1kb", "hello.cpp"));
    assert!(has("size:>1kb", "big.txt"));
    assert!(has("size:<1kb", "world.h"));
    assert!(!has("size:<1kb", "big.txt"));

    assert!(has("path:subdir", "buried.h"));
    assert!(!has("path:subdir", "hello.cpp"));
    assert!(has("nopath:subdir", "hello.cpp"));

    let parent = format!(
        "parent:{}",
        root.join("subdir").to_string_lossy().replace('\\', "/")
    );
    assert!(has(&parent, "nested.cpp"));
    assert!(!has(&parent, "buried.h"));

    // Hits carry real metadata and a usable full path.
    let hit = e
        .search("hello.cpp", 10)
        .into_iter()
        .find(|h| h.name == "hello.cpp")
        .unwrap();
    assert_eq!(hit.size, 1025);
    assert!(hit.modified > 0);
    assert!(PathBuf::from(hit.full_path()).exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn index_round_trip() {
    let root = fixture("roundtrip");
    let mut e = Engine::new();
    e.scan(&root);
    let file = root.join("index.bin");
    assert!(e.save(&file));
    let mut loaded = Engine::new();
    assert!(loaded.load(&file));
    assert_eq!(loaded.len(), e.len());
    assert_eq!(names(&loaded, "ext:cpp").len(), names(&e, "ext:cpp").len());
    let _ = fs::remove_dir_all(&root);
}
