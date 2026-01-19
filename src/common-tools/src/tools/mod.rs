use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::tool_box;

use std::path::{Path, PathBuf};

pub mod apply_patch;
pub mod copy_path;
pub mod create_directory;
pub mod execute_command;
pub mod list_directory;
pub mod move_path;
pub mod now;
pub mod read_file;
pub mod search_replace_edit;
pub mod task_complete;
pub mod write_file;

pub use apply_patch::ApplyPatchTool;
pub use copy_path::CopyPathTool;
pub use create_directory::CreateDirectoryTool;
pub use execute_command::ExecuteCommandTool;
pub use list_directory::ListDirectoryTool;
pub use move_path::MovePathTool;
pub use now::NowTool;
pub use read_file::ReadFileTool;
pub use search_replace_edit::{EditOperation, SearchReplaceEditTool};
pub use task_complete::TaskCompleteTool;
pub use write_file::WriteFileTool;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Tool error: {0}")]
    Other(String),
}

impl From<ToolError> for CallToolError {
    fn from(error: ToolError) -> Self {
        CallToolError::from_message(error.to_string())
    }
}

pub fn resolve_path(path: &str, working_directory: Option<&Path>) -> PathBuf {
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        path_obj.to_path_buf()
    } else if let Some(wd) = working_directory {
        wd.join(path_obj)
    } else {
        std::env::current_dir().unwrap_or_default().join(path_obj)
    }
}

tool_box!(
    CommonTools,
    [
        ReadFileTool,
        WriteFileTool,
        ExecuteCommandTool,
        ListDirectoryTool,
        CreateDirectoryTool,
        CopyPathTool,
        MovePathTool,
        NowTool,
        SearchReplaceEditTool,
        ApplyPatchTool,
        TaskCompleteTool
    ]
);
