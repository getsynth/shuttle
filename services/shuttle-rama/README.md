## Shuttle service integration for the Rama framework

Rama is still in early development and for now the latest
alpha release is used, `0.2.0-alpha.5`.

### Examples

#### Application Service

```rust,ignore
use rama::service::service_fn;
use std::convert::Infallible;

async fn hello_world() -> Result<&'static str, Infallible> {
    Ok("Hello, world!")
}

#[shuttle_runtime::main]
async fn main() -> Result<impl shuttle_rama::ShuttleService, shuttle_rama::ShuttleError> {
    Ok(shuttle_rama::RamaService::application(
        service_fn(hello_world),
    ))
}
```

#### Transport Service

```rust,ignore
use rama::{net, service::service_fn};
use std::convert::Infallible;
use tokio::io::AsyncWriteExt;

async fn hello_world<S>(mut stream: S) -> Result<(), Infallible>
where
    S: net::stream::Socket + net::stream::Stream + Unpin,
{
    println!(
        "Incoming connection from: {}",
        stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "???".to_owned())
    );

    const TEXT: &str = "Hello, Shuttle!";

    let resp = [
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        format!("Content-Length: {}", TEXT.len()).as_str(),
        "",
        TEXT,
        "",
    ]
    .join("\r\n");

    stream
        .write_all(resp.as_bytes())
        .await
        .expect("write to stream");

    Ok::<_, std::convert::Infallible>(())
}

#[shuttle_runtime::main]
async fn main() -> Result<impl shuttle_rama::ShuttleService, shuttle_rama::ShuttleError> {
    Ok(shuttle_rama::RamaService::transport(service_fn(
        hello_world,
    )))
}
```
