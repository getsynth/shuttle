use shuttle_common::service;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub user_id: String,
}

impl From<Service> for service::Response {
    fn from(service: Service) -> Self {
        Self {
            id: service.id,
            name: service.name,
            user_id: service.user_id,
        }
    }
}
