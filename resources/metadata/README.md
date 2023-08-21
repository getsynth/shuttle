# Shuttle Metadata

This plugin allows applications to obtain certain information about their runtime environment.

## Usage

Add `shuttle-metadata` to the dependencies for your service.

You can get this resource using the `shuttle_metadata::ShuttleMetadata` attribute to get a `Metadata`. This struct will contain information such as the Shuttle service name.

```rust
#[shuttle_runtime::main]
async fn app(
    #[shuttle_metadata::ShuttleMetadata] metadata: shuttle_metadata::Metadata,
) -> __ { ... }
```

#### Example projects that use `shuttle-metadata`

| Framework | Link                                                                                   |
| --------- | -------------------------------------------------------------------------------------- |
| Axum      | [axum example](https://github.com/shuttle-hq/shuttle-examples/tree/main/axum/metadata) |
