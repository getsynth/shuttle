struct MyService;

#[shuttle_service::async_trait]
impl shuttle_service::Service for MyService {
    async fn bind(mut self, _: std::net::SocketAddr) -> Result<(), shuttle_service::Error> {
        panic!("panic in bind");
    }
}

#[shuttle_service::main]
async fn bind_panic() -> Result<MyService, shuttle_service::Error> {
    Ok(MyService)
}
