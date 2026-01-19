use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "move_path",
    description = "Move or rename a file or directory to a new location"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MovePathTool {
    /// The path to the file or directory to move (absolute or relative to working directory)
    pub source_path: String,
    /// The destination path (absolute or relative to working directory)
    pub destination_path: String,
}

impl MovePathTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let source_abs = resolve_path(&self.source_path, None);
        let dest_abs = resolve_path(&self.destination_path, None);

        if !source_abs.exists() {
            return Err(ToolError::FileNotFound(source_abs.display().to_string()).into());
        }

        if let Some(parent) = dest_abs.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(ToolError::Io)?;
            }
        }

        fs::rename(&source_abs, &dest_abs).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(format!(
                    "{}:{}",
                    source_abs.display(),
                    dest_abs.display()
                ))
            } else {
                ToolError::Io(e)
            }
        })?;

        let message = format!("Successfully moved to {}", dest_abs.display());
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
