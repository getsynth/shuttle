use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Response {
    pub name: String,
    pub key: String,
    pub account_tier: String,
    pub subscription_id: Option<String>,
}
