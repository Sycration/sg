//! Runs the Python tests in tests/python against the Python module: sggeo built with its `python` feature, in
//! a target directory of its own, and loaded from a directory next to the tests. Without Python 3 and SymPy
//! the test passes with a notice, unless SGGEO_REQUIRE_PYTHON is set.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn python() {
    let python = std::env::var("PYTHON").unwrap_or_else(|_| "python3".into());
    if !Command::new(&python).args(["-c", "import sympy"]).output().is_ok_and(|o| o.status.success()) {
        assert!(std::env::var_os("SGGEO_REQUIRE_PYTHON").is_none(), "Python 3 ({python}) with SymPy is not installed");
        eprintln!("skipping the Python tests: Python 3 ({python}) with SymPy is not installed");
        return;
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let target = tmp.join("sggeo-python-target");
    let release = !cfg!(debug_assertions);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut build = Command::new(cargo);
    build.args(["build", "-p", "sggeo", "--lib", "--features", "python", "--target-dir"]).arg(&target).current_dir(&manifest);
    if release {
        build.arg("--release");
    }
    let out = build.output().unwrap();
    assert!(out.status.success(), "building the Python module failed:\n{}", String::from_utf8_lossy(&out.stderr));

    let (lib, module) = match std::env::consts::OS {
        "windows" => ("sggeo.dll", "sggeo.pyd"),
        "macos" => ("libsggeo.dylib", "sggeo.abi3.so"),
        _ => ("libsggeo.so", "sggeo.abi3.so"),
    };
    let dir = tmp.join("sggeo-python");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(target.join(if release { "release" } else { "debug" }).join(lib), dir.join(module)).unwrap();

    let out = Command::new(&python)
        .args(["-m", "unittest", "-v", "test_sggeo"])
        .current_dir(manifest.join("tests/python"))
        .env("PYTHONPATH", &dir)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("SGGEO_EXAMPLES", manifest.join("test_examples"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "the Python tests failed:\n{text}");
    eprintln!("{text}");
}
