use chrono::{DateTime, Local, Utc};
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "now",
    description = "Get the current date and time in ISO 8601 format"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct NowTool {
    /// Timezone for the datetime: 'utc' or 'local' (default: 'local')
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "local".to_string()
}

impl NowTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let (iso_datetime, tz_name) = match self.timezone.as_str() {
            "utc" => {
                let now: DateTime<Utc> = Utc::now();
                (now.to_rfc3339(), "utc")
            }
            "local" | _ => {
                let now: DateTime<Local> = Local::now();
                (now.to_rfc3339(), "local")
            }
        };

        let message = format!("Current time ({}): {}", tz_name, iso_datetime);
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
