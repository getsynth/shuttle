use shuttle_service::error::CustomError;
use shuttle_service::{log, IntoService, ResourceBuilder, Runtime, ServeHandle, Service};
use sqlx::PgPool;

#[macro_use]
extern crate shuttle_service;

struct Args;

struct PoolService {
    runtime: Runtime,
    pool: Option<PgPool>,
}

fn init() -> Args {
    Args
}

impl IntoService for Args {
    type Service = PoolService;

    fn into_service(self) -> Self::Service {
        PoolService {
            pool: None,
            runtime: Runtime::new().unwrap(),
        }
    }
}

async fn start(pool: PgPool) -> Result<(), shuttle_service::error::CustomError> {
    let (rec,): (String,) = sqlx::query_as("SELECT 'Hello world'")
        .fetch_one(&pool)
        .await
        .map_err(CustomError::new)?;

    assert_eq!(rec, "Hello world");

    Ok(())
}

#[async_trait]
impl Service for PoolService {
    fn bind(
        &mut self,
        _: std::net::SocketAddr,
    ) -> Result<ServeHandle, shuttle_service::error::Error> {
        let launch = start(self.pool.take().expect("we should have an active pool"));
        let handle = self.runtime.spawn(launch);

        Ok(handle)
    }

    async fn build(
        &mut self,
        factory: &mut dyn shuttle_service::Factory,
        logger: Box<dyn log::Log>,
    ) -> Result<(), shuttle_service::Error> {
        self.runtime
            .spawn_blocking(move || {
                shuttle_service::log::set_boxed_logger(logger)
                    .map(|()| {
                        shuttle_service::log::set_max_level(shuttle_service::log::LevelFilter::Info)
                    })
                    .expect("logger set should succeed");
            })
            .await
            .unwrap();

        let pool = shuttle_service::shared::Postgres::new()
            .build(factory, &self.runtime)
            .await?;

        self.pool = Some(pool);

        Ok(())
    }
}

declare_service!(Args, init);
