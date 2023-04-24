use chrono::{DateTime, Utc};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment,
    ContentArrangement, Table,
};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = shuttle_common::models::secret::Response)]
pub struct Response {
    pub key: String,
    #[schema(value_type = KnownFormat::DateTime)]
    pub last_update: DateTime<Utc>,
}

pub fn get_table(secrets: &Vec<Response>) -> String {
    if secrets.is_empty() {
        format!("{}\n", "No secrets are linked to this service".bold())
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(vec![
                Cell::new("Key")
                    .set_alignment(CellAlignment::Center)
                    .add_attribute(Attribute::Bold),
                Cell::new("Last updated")
                    .set_alignment(CellAlignment::Center)
                    .add_attribute(Attribute::Bold),
            ]);

        for resource in secrets.iter() {
            table.add_row(vec![
                resource.key.to_string(),
                resource
                    .last_update
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ]);
        }

        format!(
            r#"These secrets are linked to this service
{table}
"#,
        )
    }
}
