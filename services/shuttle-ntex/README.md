## Shuttle service integration for the Ntex Web framework

### Example

```rust,no_run
use ntex::web::{get, ServiceConfig};
use shuttle_ntex::ShuttleNtexWeb;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}

#[shuttle_runtime::main]
async fn ntex_web() -> ShuttleNtexWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world);
    };

    Ok(config.into())
}
```
