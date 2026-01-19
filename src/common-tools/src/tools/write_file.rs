use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "write_file",
    description = "Write content to a text file on the file system"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WriteFileTool {
    /// The path to the file to write (absolute or relative to working directory)
    pub path: String,
    /// The text content to write to the file
    pub content: String,
}

impl WriteFileTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let abs_path = resolve_path(&self.path, None);

        if let Some(parent) = abs_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        ToolError::PermissionDenied(parent.display().to_string())
                    } else {
                        ToolError::Io(e)
                    }
                })?;
            }
        }

        fs::write(&abs_path, &self.content).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        let message = format!("Successfully wrote to {}", abs_path.display());
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
