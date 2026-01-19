use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};

use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "read_file",
    description = "Read the contents of a text file from the file system"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReadFileTool {
    /// The path to the file to read (absolute or relative to working directory)
    pub path: String,
}

impl ReadFileTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let abs_path = resolve_path(&self.path, None);

        if !abs_path.exists() {
            return Err(ToolError::FileNotFound(abs_path.display().to_string()).into());
        }

        if !abs_path.is_file() {
            return Err(ToolError::InvalidArgument(format!(
                "Path is not a file: {}",
                abs_path.display()
            ))
            .into());
        }

        let content = fs::read_to_string(&abs_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            content,
        )]))
    }
}
