use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenMessage {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct KeyMessage {
    pub api_key: String,
}
