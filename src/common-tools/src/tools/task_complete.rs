use rmcp::model::{CallToolResult, Content};

use super::ToolError;

pub struct TaskCompleteTool {
    pub task_id: String,
    pub result: String,
}

impl TaskCompleteTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let message = format!("Task {} marked as complete: {}", self.task_id, self.result);
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}
