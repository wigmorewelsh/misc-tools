use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use super::{resolve_path, ToolError};

#[mcp_tool(
    name = "copy_path",
    description = "Copy a file or directory to a new location"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CopyPathTool {
    /// The path to the file or directory to copy (absolute or relative to working directory)
    pub source_path: String,
    /// The destination path (absolute or relative to working directory)
    pub destination_path: String,
}

impl CopyPathTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let source_abs = resolve_path(&self.source_path, None);
        let dest_abs = resolve_path(&self.destination_path, None);

        if !source_abs.exists() {
            return Err(ToolError::FileNotFound(source_abs.display().to_string()).into());
        }

        if source_abs.is_dir() {
            copy_dir_all(&source_abs, &dest_abs).await?;
            let message = format!("Successfully copied directory to {}", dest_abs.display());
            Ok(CallToolResult::text_content(vec![TextContent::from(
                message,
            )]))
        } else {
            if let Some(parent) = dest_abs.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).await.map_err(ToolError::Io)?;
                }
            }

            fs::copy(&source_abs, &dest_abs).await.map_err(|e| {
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

            let message = format!("Successfully copied file to {}", dest_abs.display());
            Ok(CallToolResult::text_content(vec![TextContent::from(
                message,
            )]))
        }
    }
}

fn copy_dir_all<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
    Box::pin(async move {
        if !dst.exists() {
            fs::create_dir_all(dst).await?;
        }

        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_all(&src_path, &dest_path).await?;
            } else {
                fs::copy(&src_path, &dest_path).await?;
            }
        }

        Ok(())
    })
}
