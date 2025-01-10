## Shuttle service integration for the Axum web framework

Axum 0.8 is used by default.

Axum 0.7 is supported by using these feature flags:

```toml,ignore
axum = "0.7.3"
shuttle-axum = { version = "...", default-features = false, features = ["axum-0-7"] }
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
