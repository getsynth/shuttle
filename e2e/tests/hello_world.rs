use std::process::Command;
use std::str;

use tempdir::TempDir;

#[test]
fn hello() {
    let tmp_dir = TempDir::new("e2e").unwrap();

    // Spawn into background
    let mut api_process = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "api",
            "--",
            "--path",
            tmp_dir.path().to_str().unwrap(),
        ])
        .env("PG_PASSWORD", "super-secret-pwd")
        .current_dir("../")
        .spawn()
        .unwrap();

    let unveil_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "cargo-unveil",
            "--manifest-path",
            "../../../Cargo.toml",
            "--",
            "deploy",
        ])
        .current_dir("../examples/rocket/hello-world")
        .output()
        .unwrap();

    let stdout = str::from_utf8(&unveil_output.stdout).unwrap();
    assert!(
        stdout.contains("Finished dev"),
        "output does not contain 'Finished dev':\nstdout = {}\nstderr = {}",
        stdout,
        str::from_utf8(&unveil_output.stderr).unwrap()
    );
    assert!(stdout.contains("Deployment Status:  DEPLOYED"));

    let request_text = reqwest::blocking::Client::new()
        .get("http://localhost:8000/hello")
        .header("Host", "hello-world-rocket-app.unveil.sh")
        .send()
        .unwrap()
        .text()
        .unwrap();

    assert_eq!(request_text, "Hello, world!");

    api_process.kill().unwrap();
}
