## How to run

```bash
$ make wasm
$ DISCORD_TOKEN=xxx cargo run
```

In another terminal:

``` bash
grpcurl -plaintext -import-path ../proto -proto runtime.proto -d '{"service_name": "Tonic", "path": "runtime/bot.wasm"}' localhost:8000 runtime.Runtime/load
grpcurl -plaintext -import-path ../proto -proto runtime.proto -d '{"service_name": "Tonic"}' localhost:8000 runtime.Runtime/start
```
## shuttle-legacy

Load and run an .so library that implements `shuttle_service::Service`. 

To test, first start this binary using:

```bash
cargo run -- --legacy
```

Then in another shell, load a `.so` file and start it up:

``` bash
grpcurl -plaintext -import-path ../proto -proto runtime.proto -d '{"service_name": "Tonic", "path": "examples/rocket/hello-world/target/debug/libhello_world.so"}' localhost:8000 runtime.Runtime/load
grpcurl -plaintext -import-path ../proto -proto runtime.proto -d '{"service_name": "Tonic"}' localhost:8000 runtime.Runtime/start
```

## Running the tests
```bash
$ cd ..; make test
```
