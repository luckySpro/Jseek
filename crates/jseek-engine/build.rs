//! Builds the C++ engine (engine/, via CMake) as a static library and links it.
//! No build-dependency crates: CMake and pkg-config are invoked directly.
use std::path::PathBuf;
use std::process::Command;

fn run(cmd: &mut Command) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to run {cmd:?}: {e}"));
    assert!(status.success(), "command failed: {cmd:?}");
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let engine = manifest
        .join("../../engine")
        .canonicalize()
        .expect("engine/ directory");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("cmake-build");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed={}", engine.join("Core").display());
    println!("cargo:rerun-if-changed={}", engine.join("Bridge").display());
    println!(
        "cargo:rerun-if-changed={}",
        engine.join("CMakeLists.txt").display()
    );
    println!("cargo:rerun-if-env-changed=JSEEK_CMAKE_ARGS");

    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(&engine)
        .arg("-B")
        .arg(&out)
        .arg("-DJSEEK_BUILD_TESTS=OFF")
        .arg("-DCMAKE_BUILD_TYPE=Release");
    if let Ok(extra) = std::env::var("JSEEK_CMAKE_ARGS") {
        configure.args(extra.split_whitespace());
    }
    run(&mut configure);
    run(Command::new("cmake")
        .arg("--build")
        .arg(&out)
        .arg("--config")
        .arg("Release")
        .arg("--target")
        .arg("jseek_engine")
        .arg("-j")
        .arg(std::env::var("NUM_JOBS").unwrap_or_else(|_| "4".into())));

    println!("cargo:rustc-link-search=native={}", out.display());
    println!(
        "cargo:rustc-link-search=native={}",
        out.join("Release").display()
    );
    println!("cargo:rustc-link-lib=static=jseek_engine");

    // re2 (and its abseil deps) via pkg-config; JSEEK_RE2_LIBS overrides, e.g. for vcpkg.
    let re2_libs = std::env::var("JSEEK_RE2_LIBS").ok().or_else(|| {
        Command::new("pkg-config")
            .args(["--libs", "re2"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    });
    match re2_libs {
        Some(flags) => {
            for f in flags.split_whitespace() {
                if let Some(dir) = f.strip_prefix("-L") {
                    println!("cargo:rustc-link-search=native={dir}");
                } else if let Some(lib) = f.strip_prefix("-l") {
                    println!("cargo:rustc-link-lib={lib}");
                }
            }
        }
        None => println!("cargo:rustc-link-lib=re2"),
    }

    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=framework=CoreServices");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=pthread");
        }
        _ => {}
    }
}
