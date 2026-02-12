use rmcp::model::{CallToolResult, Content};
use std::process::Stdio;
use tokio::process::Command;

use super::ToolError;

pub struct ExecuteCommandTool {
    pub command: String,
}

impl ExecuteCommandTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
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

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}
