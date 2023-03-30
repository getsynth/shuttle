use std::path::{Path, PathBuf};

use shuttle_service::builder::{build_workspace, Runtime};

#[tokio::test]
#[should_panic(expected = "1 job failed")]
async fn not_shuttle() {
    let (tx, _) = crossbeam_channel::unbounded();
    let project_path = format!("{}/tests/resources/not-shuttle", env!("CARGO_MANIFEST_DIR"));
    build_workspace(Path::new(&project_path), false, tx)
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Your Shuttle project must be a binary.")]
async fn not_bin() {
    let (tx, _) = crossbeam_channel::unbounded();
    let project_path = format!("{}/tests/resources/not-bin", env!("CARGO_MANIFEST_DIR"));
    match build_workspace(Path::new(&project_path), false, tx).await {
        Ok(_) => {}
        Err(e) => panic!("{}", e.to_string()),
    }
}

#[tokio::test]
async fn is_bin() {
    let (tx, _) = crossbeam_channel::unbounded();
    let project_path = format!("{}/tests/resources/is-bin", env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        build_workspace(Path::new(&project_path), false, tx)
            .await
            .unwrap(),
        vec![Runtime::Alpha(
            PathBuf::from(project_path).join("target/debug/is-bin")
        )]
    );
}

#[tokio::test]
#[should_panic(expected = "failed to read")]
async fn not_found() {
    let (tx, _) = crossbeam_channel::unbounded();
    let project_path = format!(
        "{}/tests/resources/non-existing",
        env!("CARGO_MANIFEST_DIR")
    );
    build_workspace(Path::new(&project_path), false, tx)
        .await
        .unwrap();
}

// Test that alpha and next projects are compiled correctly. Any shared library crates should not be compiled too
#[tokio::test]
async fn workspace() {
    let (tx, _) = crossbeam_channel::unbounded();
    let project_path = format!("{}/tests/resources/workspace", env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        build_workspace(Path::new(&project_path), false, tx)
            .await
            .unwrap(),
        vec![
            Runtime::Alpha(PathBuf::from(&project_path).join("target/debug/alpha")),
            Runtime::Next(PathBuf::from(&project_path).join("target/wasm32-wasi/debug/next.wasm"))
        ]
    );
}
