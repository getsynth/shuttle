use anyhow::Context;

use crate::{
    __internals::{Loader, Runner},
    alpha, rt,
};

#[derive(Default)]
struct Args {
    /// Enable compatibility with beta platform [env: SHUTTLE_BETA]
    beta: bool,
    /// Alpha (required): Port to open gRPC server on
    port: Option<u16>,
}

impl Args {
    // uses simple arg parsing logic instead of clap to reduce dependency weight
    fn parse() -> anyhow::Result<Self> {
        let mut args = Self::default();

        // The first argument is the path of the executable
        let mut args_iter = std::env::args().skip(1);

        while let Some(arg) = args_iter.next() {
            if arg.as_str() == "--port" {
                let port = args_iter
                    .next()
                    .context("missing port value")?
                    .parse()
                    .context("invalid port value")?;
                args.port = Some(port);
            }
        }

        args.beta = std::env::var("SHUTTLE_BETA").is_ok();

        if args.beta {
            if std::env::var("SHUTTLE_ENV").is_err() {
                return Err(anyhow::anyhow!(
                    "SHUTTLE_ENV is required to be set on shuttle.dev"
                ));
            }
        } else if args.port.is_none() {
            return Err(anyhow::anyhow!("--port is required"));
        }

        Ok(args)
    }
}

pub async fn start(loader: impl Loader + Send + 'static, runner: impl Runner + Send + 'static) {
    // `--version` overrides any other arguments. Used by cargo-shuttle to check compatibility on local runs.
    if std::env::args().any(|arg| arg == "--version") {
        println!("{}", crate::VERSION_STRING);
        return;
    }

    println!("{} starting", crate::VERSION_STRING);

    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("ERROR: Runtime failed to parse args: {e}");
            let help_str = "[HINT]: Run your Shuttle app with `shuttle run` or `cargo shuttle run`";
            let wrapper_str = "-".repeat(help_str.len());
            eprintln!("{wrapper_str}\n{help_str}\n{wrapper_str}");
            return;
        }
    };

    // this is handled after arg parsing to not interfere with --version above
    #[cfg(feature = "setup-tracing")]
    {
        use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
        registry()
            .with(fmt::layer().without_time())
            .with(
                // let user override RUST_LOG in local run if they want to
                EnvFilter::try_from_default_env()
                    // otherwise use our default
                    .or_else(|_| {
                        EnvFilter::try_new(if args.beta {
                            "info"
                        } else {
                            "info,shuttle=trace"
                        })
                    })
                    .unwrap(),
            )
            .init();

        if args.beta {
            tracing::warn!(
                "Default tracing subscriber initialized (https://docs.shuttle.dev/docs/logs)"
            );
        } else {
            tracing::warn!("Default tracing subscriber initialized (https://docs.shuttle.rs/configuration/logs)");
        }
    }

    if args.beta {
        rt::start(loader, runner).await
    } else {
        alpha::start(args.port.unwrap(), loader, runner).await
    }
}
