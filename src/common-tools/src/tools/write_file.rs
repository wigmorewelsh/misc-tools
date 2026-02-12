use rmcp::model::{CallToolResult, Content};
use tokio::fs;

use super::{resolve_path, ToolError};

pub struct WriteFileTool {
    pub path: String,
    pub content: String,
}

impl WriteFileTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
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
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}
