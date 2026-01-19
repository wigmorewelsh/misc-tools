use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "task_complete",
    description = "Signal that the current task has been completed successfully"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TaskCompleteTool {
    /// Brief summary of what was accomplished
    pub summary: String,
}

impl TaskCompleteTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let message = format!("Task marked as complete: {}", self.summary);
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
