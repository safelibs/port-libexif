use std::path::PathBuf;
use std::process::Command;

#[test]
fn copied_original_c_tests_pass() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join("tests").join("run-c-test.sh");
    let output = Command::new("bash")
        .arg(&script)
        .output()
        .expect("failed to run C test driver");

    assert!(
        output.status.success(),
        "C test driver failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
