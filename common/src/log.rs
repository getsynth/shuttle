#[cfg(feature = "display")]
use std::fmt::Write;

use chrono::{DateTime, Utc};
#[cfg(feature = "display")]
use crossterm::style::{StyledContent, Stylize};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

use crate::deployment::State;

pub const STATE_MESSAGE: &str = "NEW STATE";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = shuttle_common::log::Item))]
pub struct Item {
    #[cfg_attr(feature = "openapi", schema(value_type = KnownFormat::Uuid))]
    pub id: Uuid,
    #[cfg_attr(feature = "openapi", schema(value_type = KnownFormat::DateTime))]
    pub timestamp: DateTime<Utc>,
    #[cfg_attr(feature = "openapi", schema(value_type = shuttle_common::deployment::State))]
    pub state: State,
    #[cfg_attr(feature = "openapi", schema(value_type = shuttle_common::log::Level))]
    pub level: Level,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub target: String,
    pub fields: Vec<u8>,
}

#[cfg(feature = "display")]
impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let datetime: chrono::DateTime<chrono::Local> = DateTime::from(self.timestamp);

        let message = match serde_json::from_slice(&self.fields).unwrap() {
            serde_json::Value::String(str_value) if str_value == STATE_MESSAGE => {
                writeln!(f)?;
                format!("Entering {} state", self.state)
                    .bold()
                    .blue()
                    .to_string()
            }
            serde_json::Value::Object(map) => {
                let mut simple = None;
                let mut extra = vec![];

                for (key, value) in map.iter() {
                    match key.as_str() {
                        "message" => simple = value.as_str(),
                        _ => extra.push(format!("{key}={value}")),
                    }
                }

                let mut output = if extra.is_empty() {
                    String::new()
                } else {
                    format!("{{{}}} ", extra.join(" "))
                };

                if !self.target.is_empty() {
                    let target = format!("{}:", self.target).dim();
                    write!(output, "{target} ")?;
                }

                if let Some(msg) = simple {
                    write!(output, "{msg}")?;
                }

                output
            }
            other => other.to_string(),
        };

        write!(
            f,
            "{} {} {}",
            datetime.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string().dim(),
            self.level.get_colored(),
            message
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = shuttle_common::log::Level))]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(feature = "display")]
impl Level {
    fn get_colored(&self) -> StyledContent<&str> {
        match self {
            Level::Trace => "TRACE".magenta(),
            Level::Debug => "DEBUG".blue(),
            Level::Info => " INFO".green(),
            Level::Warn => " WARN".yellow(),
            Level::Error => "ERROR".red(),
        }
    }
}

impl From<&tracing::Level> for Level {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::ERROR => Self::Error,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::INFO => Self::Info,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::TRACE => Self::Trace,
        }
    }
}
