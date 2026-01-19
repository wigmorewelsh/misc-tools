use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

use super::ToolError;

#[mcp_tool(
    name = "execute_command",
    description = "Execute a shell command in a terminal and return the output"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExecuteCommandTool {
    /// The command to execute
    pub command: String,
}

impl ExecuteCommandTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &self.command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &self.command]);
            c
        };

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let output = cmd
            .output()
            .await
            .map_err(|e| ToolError::CommandFailed(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n--- STDERR ---\n");
            }
            result.push_str(&stderr);
        }

        if !output.status.success() {
            return Err(ToolError::CommandFailed(format!(
                "Command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                result
            ))
            .into());
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            result,
        )]))
    }
}
