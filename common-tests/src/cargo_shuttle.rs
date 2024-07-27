use cargo_shuttle::{Command, ProjectArgs, RunArgs, Shuttle, ShuttleArgs};
use portpicker::pick_unused_port;
use std::{fs::canonicalize, process::exit, time::Duration};
use tokio::time::sleep;

/// creates a `cargo-shuttle` run instance with some reasonable defaults set.
pub async fn cargo_shuttle_run(working_directory: &str, external: bool) -> String {
    let working_directory = match canonicalize(working_directory) {
        Ok(wd) => wd,
        Err(e) => {
            // DEBUG CI (no such file): SLEEP AND TRY AGAIN?
            println!(
                "Did not find directory: {} !!! because {:?}",
                working_directory, e
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            canonicalize(working_directory).unwrap()
        }
    };

    let port = pick_unused_port().unwrap();

    let url = if !external {
        format!("http://localhost:{port}")
    } else {
        format!("http://0.0.0.0:{port}")
    };

    let run_args = RunArgs {
        port,
        external,
        release: false,
        raw: false,
        quiet: false,
        secret_args: Default::default(),
    };

    let runner = Shuttle::new().unwrap().run(
        ShuttleArgs {
            api_url: Some("http://shuttle.invalid:80".to_string()),
            project_args: ProjectArgs {
                working_directory: working_directory.clone(),
                name: None,
            },
            offline: false,
            debug: false,
            beta: false,
            cmd: Command::Run(run_args),
        },
        false,
    );

    tokio::spawn({
        let working_directory = working_directory.clone();
        async move {
            sleep(Duration::from_secs(10 * 60)).await;

            println!(
                "run test for '{}' took too long. Did it fail to shutdown?",
                working_directory.display()
            );
            exit(1);
        }
    });

    let runner_handle = tokio::spawn(runner);

    // Wait for service to be responsive
    let mut counter = 0;
    let client = reqwest::Client::new();
    while client.get(url.clone()).send().await.is_err() {
        if runner_handle.is_finished() {
            println!(
                "run test for '{}' exited early. Did it fail to compile/run?",
                working_directory.clone().display()
            );
            exit(1);
        }

        // reduce spam
        if counter == 0 {
            println!(
                "waiting for '{}' to start up...",
                working_directory.display()
            );
        }
        counter = (counter + 1) % 10;

        sleep(Duration::from_millis(500)).await;
    }

    url
}
