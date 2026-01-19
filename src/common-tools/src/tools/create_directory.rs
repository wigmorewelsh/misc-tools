use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "create_directory",
    description = "Create a new directory at the specified path, creating parent directories as needed"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateDirectoryTool {
    /// The path of the directory to create (absolute or relative to working directory)
    pub path: String,
}

impl CreateDirectoryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let abs_path = resolve_path(&self.path, None);

        if abs_path.exists() && abs_path.is_file() {
            return Err(ToolError::InvalidArgument(format!(
                "A file already exists at path: {}",
                abs_path.display()
            ))
            .into());
        }

        fs::create_dir_all(&abs_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        let message = format!("Successfully created directory at {}", abs_path.display());
        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
