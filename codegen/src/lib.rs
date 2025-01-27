mod shuttle_main;

/// Helper macro that generates the entrypoint required by any service - likely the only macro you need in this crate.
///
/// ## Without shuttle managed resources
/// The simplest usage is when your service does not require any shuttle managed resources, so you only need to return a shuttle supported service:
///
/// ```rust,no_run
/// use shuttle_rocket::ShuttleRocket;
///
/// #[shuttle_rocket::main]
/// async fn rocket() -> ShuttleRocket {
///     let rocket = rocket::build();
///
///     Ok(rocket.into())
/// }
/// ```
///
/// ## Shuttle supported services
/// The following types can be returned from a `#[shuttle_service::main]` function and enjoy first class service support in shuttle.
///
/// | Return type       | Crate                                                          | Service                                                                          | Version    | Example                                                                                 |
/// | ----------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------- |
/// | `ShuttleActixWeb` | [shuttle-actix-web](https://crates.io/crates/shuttle-actix-web)| [actix-web](https://docs.rs/actix-web/4.3)                                       | 4.3        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/actix-web/hello-world)|
/// | `ShuttleAxum`     | [shuttle-axum](https://crates.io/crates/shuttle-axum)          | [axum](https://docs.rs/axum/0.7)                                                 | 0.7        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/axum/hello-world)     |
/// | `ShuttlePoem`     | [shuttle-poem](https://crates.io/crates/shuttle-poem)          | [poem](https://docs.rs/poem/2.0)                                                 | 2.0        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/poem/hello-world)     |
/// | `ShuttleRama`     | [shuttle-rama](https://crates.io/crates/shuttle-rama)          | [rama](https://docs.rs/rama/0.2.0-alpha.5)
/// | `ShuttleRocket`   | [shuttle-rocket](https://crates.io/crates/shuttle-rocket)      | [rocket](https://docs.rs/rocket/0.5)                                             | 0.5        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/rocket/hello-world)   |
/// | `ShuttleSalvo`    | [shuttle-salvo](https://crates.io/crates/shuttle-salvo)        | [salvo](https://docs.rs/salvo/0.63)                                              | 0.63       | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/salvo/hello-world)    |
/// | `ShuttleSerenity` | [shuttle-serenity](https://crates.io/crates/shuttle-serenity)  | [serenity](https://docs.rs/serenity/0.12) and [poise](https://docs.rs/poise/0.6) | 0.12       | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/serenity/hello-world) |
/// | `ShuttleThruster` | [shuttle-thruster](https://crates.io/crates/shuttle-thruster)  | [thruster](https://docs.rs/thruster/1.3)                                         | 1.3        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/thruster/hello-world) |
/// | `ShuttleTower`    | [shuttle-tower](https://crates.io/crates/shuttle-tower)        | [tower](https://docs.rs/tower/0.4)                                               | 0.4        | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/tower/hello-world)    |
/// | `ShuttleTide`     | [shuttle-tide](https://crates.io/crates/shuttle-tide)          | [tide](https://docs.rs/tide/0.16)                                                | 0.16       | [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/tide/hello-world)     |
///
/// ## Getting shuttle managed resources
/// Shuttle is able to manage resource dependencies for you. These resources are passed in as inputs to your `#[shuttle_runtime::main]` function and are configured using attributes:
/// ```rust,no_run
/// use sqlx::PgPool;
/// use shuttle_rocket::ShuttleRocket;
///
/// struct MyState(PgPool);
///
/// #[shuttle_runtime::main]
/// async fn rocket(#[shuttle_shared_db::Postgres] pool: PgPool) -> ShuttleRocket {
///     let state = MyState(pool);
///     let rocket = rocket::build().manage(state);
///
///     Ok(rocket.into())
/// }
/// ```
///
/// More [shuttle managed resources can be found here](https://github.com/shuttle-hq/shuttle/tree/main/resources)
#[proc_macro_error2::proc_macro_error]
#[proc_macro_attribute]
pub fn main(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    shuttle_main::tokens(attr, item)
}
