use rmcp::model::{CallToolResult, Content};
use tokio::fs;

use super::{resolve_path, ToolError};

pub struct ReadFileTool {
    pub path: String,
}

impl ReadFileTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let abs_path = resolve_path(&self.path, None);

        if !abs_path.exists() {
            return Err(ToolError::FileNotFound(abs_path.display().to_string()));
        }

        if !abs_path.is_file() {
            return Err(ToolError::InvalidArgument(format!(
                "Path is not a file: {}",
                abs_path.display()
            )));
        }

        let content = fs::read_to_string(&abs_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }
}
