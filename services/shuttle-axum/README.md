## Shuttle service integration for the Axum web framework

Axum 0.7 is used by default.

Axum 0.6 is supported by using these feature flags:

```toml,ignore
axum = "0.6.0"
shuttle-axum = { version = "0.50.0", default-features = false, features = ["axum-0-6"] }
```

### Example

```rust,ignore
use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
```
