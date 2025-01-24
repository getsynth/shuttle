<!-- markdownlint-disable -->
<p align="center">
<img width="300" src="https://raw.githubusercontent.com/shuttle-hq/shuttle/master/assets/logo-rectangle-transparent.png"/>
</p>
<br>
<p align="center">
  <a href="https://github.com/shuttle-hq/shuttle/search?l=rust">
    <img alt="language" src="https://img.shields.io/badge/language-Rust-orange.svg">
  </a>
  <a href="https://docs.shuttle.dev/">
    <img alt="docs" src="https://img.shields.io/badge/docs-shuttle.dev-orange">
  </a>
  <a href="https://docs.rs/shuttle-runtime">
    <img alt="crate-docs" src="https://img.shields.io/badge/docs-docs.rs-orange">
  </a>
  <a href="https://status.shuttle.rs/">
    <img alt="status" src="https://img.shields.io/badge/status-blue">
  </a>
  <a href="https://circleci.com/gh/shuttle-hq/shuttle/">
    <img alt="build status" src="https://circleci.com/gh/shuttle-hq/shuttle.svg?style=shield"/>
  </a>
  <a href="https://gurubase.io/g/shuttle">
    <img alt="Gurubase" src="https://img.shields.io/badge/Gurubase-Ask%20Shuttle%20Guru-006BFF"/>
  </a>
</p>
<p align="center">
  <a href="https://crates.io/crates/cargo-shuttle">
    <img alt="crates" src="https://img.shields.io/crates/d/cargo-shuttle">
  </a>
  <a href="https://discord.gg/shuttle">
    <img alt="discord" src="https://img.shields.io/discord/803236282088161321?logo=discord"/>
  </a>
  <a href="https://twitter.com/shuttle_dev">
    <img alt="Twitter Follow" src="https://img.shields.io/twitter/follow/shuttle_dev">
  </a>
</p>
<p align="center">
  <a href="https://console.algora.io/org/shuttle/bounties?status=open">
    <img alt="open bounties" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Fshuttle%2Fbounties%3Fstatus%3Dopen"/>
  </a>
  <a href="https://console.algora.io/org/shuttle/bounties?status=completed">
    <img alt="rewarded bounties" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Fshuttle%2Fbounties%3Fstatus%3Dcompleted"/>
  </a>
</p>
<br>

<h1 align="center">Fastest Way to Build & Ship Rust Apps</h1>
<div align="center">
Get resources and deploy your apps with a few lines of code.
</div>

<h3 align="center">Simple. &nbsp; Easy. &nbsp; Joyful.</h3>

<p align="center">
    <a href="https://github.com/shuttle-hq/shuttle/issues/new?assignees=&labels=S-Triage%2CT-Bug&projects=&template=BUG-REPORT.yml&title=%5BBug%5D%3A+">Report Bug</a>
    ·
    <a href="https://github.com/shuttle-hq/shuttle/issues/new?assignees=&labels=S-Triage%2CT-Feature+Request&projects=&template=FEATURE-SUGGESTION.yml&title=%5BFeature%5D%3A+">Request a Feature</a>
    ·
  <a href="https://discord.gg/shuttle">Join Our Discord</a>
    ·
    <a href="https://twitter.com/shuttle_dev">Follow us on Twitter</a>
  </p>

<div align="center"><img src="https://i.imgur.com/1qdWipP.gif" width="100%" ></div>
<br>


*<div align="center">⭐ If you find Shuttle interesting, consider starring this repo to help spread the word.</div>*
<br>

# Features

* __One-line Resource Provisioning:__ Get a database, or any other AWS resource by adding a line of code to your main file. To delete one, just remove that line of code. No config/yaml files required.
* __Rapid Development:__ It takes 2 minutes from project initialization to a deployed project. It takes another 2 minutes to provision a resource, and get it deployed to production.
* __First-class support for popular Rust frameworks:__ [Axum](https://docs.shuttle.dev/examples/axum), [Actix Web](https://docs.shuttle.dev/examples/actix), [Rocket](https://docs.shuttle.dev/examples/rocket), and [more](https://docs.shuttle.dev/examples/other)
*  __Security:__ Let us worry about the security & permissions while you focus on writing good code.
<br>
<br>

# Quick Start

On Linux and macOS, you can use this install script, which will automatically install the correct target for your OS and distro:

```sh
curl -sSfL https://www.shuttle.dev/install | bash
```

On Windows, you can use this install script to do the same:

```powershell
iwr "https://www.shuttle.dev/install-win" | iex
```

After installing, log in with:

```sh
shuttle login
```

To initialize your project, simply write:

```bash
shuttle init --template axum hello-world
```

And to deploy it, write:

```bash
cd hello-world
shuttle deploy
```

And... that's it!

```text
Service Name:  hello-world
Deployment ID: 3d08ac34-ad63-41c1-836b-99afdc90af9f
Status:        running
Last Updated:  2022-04-01T08:32:34Z
URI:           https://hello-world.shuttleapp.rs
```

Feel free to build on top of the generated `hello-world` boilerplate or take a stab at one of our [examples](https://github.com/shuttle-hq/shuttle-examples).

For the full documentation, visit [our docs](https://docs.shuttle.dev).
<br>
<br>

# Quick Look

Below is a basic "Hello World" application written in Axum:
```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_world));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}
```

In order to be able to deploy it with a single command, we update the snippet as follows:

```rust
use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
```

Now, with just `shuttle deploy`, you can see your application live. But let's enhance it further by adding a shared Postgres database:

```rust
use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {

    pool.execute(include_str!("../schema.sql"))
        .await
        .expect("failed to run migrations");

    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
```

Now, if we run `shuttle deploy`, we'll have an up and running project with a database inside & ready to use.
<br>
<br>

# Repositories

| Name | Description |
|-|-|
| [shuttle](https://github.com/shuttle-hq/shuttle) 🚀 (This repo) | The core Shuttle product. Contains all crates that users interact with. |
| [shuttle-examples](https://github.com/shuttle-hq/shuttle-examples) 👨‍🏫 | Officially maintained examples of projects that can be deployed on Shuttle. Also has a list of [community examples](https://github.com/shuttle-hq/shuttle-examples#community-examples). |
| [shuttle-docs](https://github.com/shuttle-hq/shuttle-docs) 📃 | Documentation hosted on [docs.shuttle.dev](https://docs.shuttle.dev/). |
| [www](https://github.com/shuttle-hq/www) 🌍 | Our website [shuttle.dev](https://www.shuttle.dev/), including the [blog](https://www.shuttle.dev/blog/tags/all) and [Launchpad newsletter](https://www.shuttle.dev/launchpad). |
| [deploy-action](https://github.com/shuttle-hq/deploy-action) ⚙ | GitHub Action for continuous deployments. |
| [awesome-shuttle](https://github.com/shuttle-hq/awesome-shuttle) 🌟 | An awesome list of Shuttle-hosted projects and resources that users can add to. |
| [shuttlings](https://github.com/shuttle-hq/shuttlings) ⚔️ | A collection of Rust code challenges. A great way to get started with using Rust and Shuttle. |
<br>
<br>

# Contributing to Shuttle

Contributing to Shuttle is highly encouraged!
Even if you are not planning to submit any code, joining our [Discord server](https://discord.gg/shuttle) and providing feedback helps us a lot!

Check out our [contributing docs](./CONTRIBUTING.md) and find the appropriate repo above to contribute to.
For development of this repo, check the [development docs](./DEVELOPING.md).

### Algora Bounties 💰

To offload work from the engineering team on low-priority issues, we will sometimes add a cash bounty to issues.
Sign up to the [Algora Console](https://console.algora.io/org/shuttle/bounties?status=open) to find open issues with bounties.

# Project Status

We are currently in Public Beta.
Watch "releases" of this repo to get notified of major updates!

- [x] Alpha: We are testing Shuttle, API and deployments may be unstable
- [x] Public Alpha: Anyone can sign up, but go easy on us,
  there are a few kinks
- [x] Public Beta: Stable enough for most non-enterprise use-cases
- [ ] Public: Production-ready!
<br>
<br>

# Contributors ✨

Thanks goes to these wonderful people:

<!-- markdownlint-disable -->
<a href="https://github.com/shuttle-hq/shuttle/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=shuttle-hq/shuttle" />
</a>

Made with [contrib.rocks](https://contrib.rocks).
