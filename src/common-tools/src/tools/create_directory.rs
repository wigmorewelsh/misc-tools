use rmcp::model::{CallToolResult, Content};
use tokio::fs;

use super::{resolve_path, ToolError};

pub struct CreateDirectoryTool {
    pub path: String,
}

impl CreateDirectoryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let abs_path = resolve_path(&self.path, None);

        if abs_path.exists() && abs_path.is_file() {
            return Err(ToolError::InvalidArgument(format!(
                "A file already exists at path: {}",
                abs_path.display()
            )));
        }

        fs::create_dir_all(&abs_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::PermissionDenied(abs_path.display().to_string())
            } else {
                ToolError::Io(e)
            }
        })?;

        let message = format!("Successfully created directory at {}", abs_path.display());
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}
