use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "list_directory",
    description = "List files and directories in a given directory path"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListDirectoryTool {
    /// The path to the directory to list (absolute or relative to working directory)
    pub path: String,
}

impl ListDirectoryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let abs_path = resolve_path(&self.path, None);

        if !abs_path.exists() {
            return Err(ToolError::FileNotFound(abs_path.display().to_string()).into());
        }

        if !abs_path.is_dir() {
            return Err(ToolError::InvalidArgument(format!(
                "Path is not a directory: {}",
                abs_path.display()
            ))
            .into());
        }

        let mut entries = fs::read_dir(&abs_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(ToolError::Io)? {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }

        names.sort();

        let content = format!(
            "Directory '{}' contains {} entries:\n{}",
            abs_path.display(),
            names.len(),
            names.join("\n")
        );

        Ok(CallToolResult::text_content(vec![TextContent::from(
            content,
        )]))
    }
}
