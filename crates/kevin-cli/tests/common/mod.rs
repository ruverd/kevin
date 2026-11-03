#![allow(dead_code)]
use serde_json::Value;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::OnceLock,
};

pub fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_kevin")
}
pub fn run(cwd: &Path, args: &[&str], input: &str) -> Output {
    run_exe(Path::new(binary()), cwd, args, input)
}
pub fn run_exe(exe: &Path, cwd: &Path, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
pub fn ok(cwd: &Path, args: &[&str], input: &str) -> String {
    let output = run(cwd, args, input);
    assert!(
        output.status.success(),
        "args: {args:?}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
pub fn value(cwd: &Path, args: &[&str], input: &str) -> Value {
    serde_json::from_str(&ok(cwd, args, input)).unwrap()
}
pub fn fixture() -> &'static Path {
    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &FIXTURE
        .get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let exe = dir
                .path()
                .join(format!("provider{}", std::env::consts::EXE_SUFFIX));
            let rustc = Path::new(env!("CARGO"))
                .with_file_name(format!("rustc{}", std::env::consts::EXE_SUFFIX));
            let source =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/provider.rs");
            let output = Command::new(rustc)
                .arg(source)
                .arg("-o")
                .arg(&exe)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            (dir, exe)
        })
        .1
}
pub fn init() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    ok(dir.path(), &["init"], "");
    dir
}
