use std::path::PathBuf;
use std::process::Command;

#[test]
fn copied_original_test_suite_passes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir
        .join("tests")
        .join("run-original-test-suite.sh");
    let output = Command::new("bash")
        .arg(&script)
        .output()
        .expect("failed to run original test suite");

    assert!(
        output.status.success(),
        "original test suite failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
