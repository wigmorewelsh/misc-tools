use chrono::{DateTime, Local, Utc};
use rmcp::model::{CallToolResult, Content};

use super::ToolError;

pub struct NowTool {
    pub timezone: String,
}

impl NowTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
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
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}
