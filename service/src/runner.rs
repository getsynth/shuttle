use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Context;
use shuttle_proto::runtime;
use tokio::process;
use tracing::info;

pub async fn start(
    port: u16,
    runtime_executable: PathBuf,
    project_path: &Path,
) -> anyhow::Result<(process::Child, runtime::Client)> {
    let port = &port.to_string();
    let args = vec!["--port", port];

    info!(
        args = %format!("{} {}", runtime_executable.display(), args.join(" ")),
        "Spawning runtime process",
    );
    let runtime = process::Command::new(
        dunce::canonicalize(runtime_executable).context("canonicalize path of executable")?,
    )
    .current_dir(project_path)
    .args(&args)
    .stdout(Stdio::piped())
    .kill_on_drop(true)
    .spawn()
    .context("spawning runtime process")?;

    let runtime_client = runtime::get_client(port).await?;

    Ok((runtime, runtime_client))
}
