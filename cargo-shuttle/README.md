<!-- markdownlint-disable -->
<div align="center">

# cargo-shuttle

<p align=center>
  <a href="https://docs.rs/shuttle-service">
    <img alt="docs" src="https://img.shields.io/badge/docs-reference-orange">
  </a>
  <a href="https://github.com/shuttle-hq/shuttle/search?l=rust">
    <img alt="language" src="https://img.shields.io/badge/language-Rust-orange.svg">
  </a>
  <a href="https://circleci.com/gh/shuttle-hq/shuttle/">
    <img alt="build status" src="https://circleci.com/gh/shuttle-hq/shuttle.svg?style=shield"/>
  </a>
  <a href="https://discord.gg/shuttle">
    <img alt="discord" src="https://img.shields.io/discord/803236282088161321?logo=discord"/>
  </a>
</p>
<!-- markdownlint-restore -->
<!-- markdownlint-disable MD001 -->

`cargo-shuttle` is your commandline tool for deploying web apps on [shuttle](https://www.shuttle.rs/), the stateful serverless web platform for Rust.

**README Sections:** [Installation](#installation) — [Subcommands](#subcommands) — [Development](#development)

</div>

---

`cargo-shuttle` brings [shuttle](https://www.shuttle.rs/), the open source serverless platform for Rust web applications, into your terminal. With a dedicated focus on productivity, reliability, and performance, `cargo-shuttle` makes deploying your code to the cloud as easy as deriving a trait.

---

<!-- markdownlint-disable-next-line -->
<a id="installation"><h1>Installation</h1></a>

`cargo-shuttle` is available for macOS, Linux, and Windows. To install the commandline tool, run:

```bash
cargo install cargo-shuttle
```

### Distro Packages

<details>
  <summary>Packaging status</summary>

[![Packaging status](https://repology.org/badge/vertical-allrepos/cargo-shuttle.svg)](https://repology.org/project/cargo-shuttle/versions)

</details>

#### Arch Linux

`cargo-shuttle` can be installed from the [community repository](https://archlinux.org/packages/community/x86_64/cargo-shuttle) using [pacman](https://wiki.archlinux.org/title/Pacman):

```sh
pacman -S cargo-shuttle
```

---

<!-- markdownlint-disable-next-line -->
<a id="subcommands"><h1>Subcommands</h1></a>

`cargo-shuttle`'s subcommands help you build and deploy web apps from start to finish.

Run `cargo shuttle help` to see the basic usage:

```text
Usage: cargo-shuttle [OPTIONS] <COMMAND>

Commands:
  init        Create a new shuttle project
  run         Run a shuttle service locally
  deploy      Deploy a shuttle service
  deployment  Manage deployments of a shuttle service
  status      View the status of a shuttle service
  stop        Stop this shuttle service
  logs        View the logs of a deployment in this shuttle service
  project     List or manage projects on shuttle
  resource    Manage resources of a shuttle project
  secrets     Manage secrets for this shuttle service
  clean       Remove cargo build artifacts in the shuttle environment
  login       Login to the shuttle platform
  logout      Log out of the shuttle platform
  generate    Generate shell completions
  feedback    Open an issue on GitHub and provide feedback
  help        Print this message or the help of the given subcommand(s)

Options:
      --working-directory <WORKING_DIRECTORY>  Specify the working directory [default: .]
      --name <NAME>                            Specify the name of the project (overrides crate name)
      --api-url <API_URL>                      Run this command against the API at the supplied URL (allows targeting a custom deployed instance for this command only, mainly
                                               for development) [env: SHUTTLE_API=]
  -h, --help                                   Print help
  -V, --version                                Print version
```

### Subcommand: `init`

To initialize a shuttle project with boilerplates, run `cargo shuttle init [OPTIONS] [PATH]`.

Currently, `cargo shuttle init` supports the following frameworks:

- `--template actix-web`: for [actix web](https://actix.rs/) framework
- `--template axum`: for [axum](https://github.com/tokio-rs/axum) framework
- `--template poem`: for [poem](https://github.com/poem-web/poem) framework
- `--template poise`: for [poise](https://github.com/serenity-rs/poise) discord bot framework
- `--template rocket`: for [rocket](https://rocket.rs/) framework
- `--template salvo`: for [salvo](https://salvo.rs/) framework
- `--template serenity`: for [serenity](https://github.com/serenity-rs/serenity) discord bot framework
- `--template thruster`: for [thruster](https://github.com/thruster-rs/Thruster) framework
- `--template tide`: for [tide](https://github.com/http-rs/tide) framework
- `--template tower`: for [tower](https://github.com/tower-rs/tower) library
- `--template warp`: for [warp](https://github.com/seanmonstar/warp) framework

For example, running the following command will initialize a project for [rocket](https://rocket.rs/):

```sh
cargo shuttle init --template rocket my-rocket-app
```

This should generate the following dependency in `Cargo.toml`:

```toml
rocket = "0.5.0-rc.2"
shuttle-rocket = { version = "0.20.0" }
shuttle-runtime = { version = "0.20.0" }
tokio = { version = "1.26.0" }
```

The following boilerplate code should be generated into `src/lib.rs`:

```rust
#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index]);

    Ok(rocket.into())
}
```

### Subcommand: `run`

To run the shuttle project locally, use the following command:

```sh
# Inside your shuttle project
cargo shuttle run
```

This will compile your shuttle project and start it on the default port `8000`. Test it by:

```sh
$ curl http://localhost:8000
Hello, world!
```

### Subcommand: `login`

Use `cargo shuttle login` inside your shuttle project to generate an API key for the shuttle platform:

```sh
# Inside a shuttle project
cargo shuttle login
```

This should automatically open a browser window with an auto-generated API key for your project. Simply copy-paste the API key back in your terminal or run the following command to complete login:

```sh
cargo shuttle login --api-key <your-api-key-from-browser>
```

### Subcommand: `deploy`

To deploy your shuttle project to the cloud, run:

```sh
cargo shuttle project start
cargo shuttle deploy
```

Your service will immediately be available at `{crate_name}.shuttleapp.rs`. For instance:

```sh
$ curl https://my-rocket-app.shuttleapp.rs
Hello, world!
```

### Subcommand: `status`

Check the status of your deployed shuttle project with:

```sh
cargo shuttle status
```

### Subcommand: `logs`

Check the logs of your deployed shuttle project with:

```sh
cargo shuttle logs
```

### Subcommand: `stop`

Once you are done with a deployment, you can stop it by running:

```sh
cargo shuttle stop
```

---

<!-- markdownlint-disable-next-line -->
<a id="development"><h1>Development</h1></a>

Thanks for using `cargo-shuttle`! We’re very happy to have you with us!

During our alpha period, API keys are completely free and you can deploy as many services as you want.

Just keep in mind that there may be some kinks that require us to take all deployments down once in a while. In certain circumstances we may also have to delete all the data associated with those deployments.

To contribute to `cargo-shuttle` or stay updated with our development, please [open an issue, discussion or PR on Github](https://github.com/shuttle-hq/shuttle) and [join our Discord](https://discord.gg/shuttle)! 🚀
