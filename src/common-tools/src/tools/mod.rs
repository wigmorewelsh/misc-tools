use std::path::{Path, PathBuf};

pub mod apply_patch;
pub mod copy_path;
pub mod create_directory;
pub mod execute_command;
pub mod grep;
pub mod list_directory;
pub mod move_path;
pub mod now;
pub mod read_file;
pub mod search_replace_edit;
pub mod task_complete;
pub mod write_file;

#[cfg(test)]
mod tests;

pub use apply_patch::ApplyPatchTool;
pub use copy_path::CopyPathTool;
pub use create_directory::CreateDirectoryTool;
pub use execute_command::ExecuteCommandTool;
pub use grep::GrepTool;
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadFileParams {
    #[schemars(
        description = "The path to the file to read (absolute or relative to working directory)"
    )]
    pub path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WriteFileParams {
    #[schemars(description = "The path to the file to write")]
    pub path: String,
    #[schemars(description = "The content to write to the file")]
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExecuteCommandParams {
    #[schemars(description = "The command to execute")]
    pub command: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GrepParams {
    #[schemars(description = "Regular expression pattern to search for")]
    pub regex: String,
    #[schemars(description = "Glob pattern to filter files (e.g., '**/*.rs')")]
    pub include_pattern: Option<String>,
    #[schemars(description = "Offset for pagination (default: 0)")]
    pub offset: Option<u32>,
    #[schemars(description = "Whether the search is case sensitive (default: false)")]
    pub case_sensitive: Option<bool>,
    #[schemars(description = "Working directory for the search")]
    pub working_directory: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListDirectoryParams {
    #[schemars(description = "The path to the directory to list")]
    pub path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateDirectoryParams {
    #[schemars(description = "The path where the directory should be created")]
    pub path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CopyPathParams {
    #[schemars(description = "The source path to copy from")]
    pub source_path: String,
    #[schemars(description = "The destination path to copy to")]
    pub destination_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MovePathParams {
    #[schemars(description = "The source path to move from")]
    pub source_path: String,
    #[schemars(description = "The destination path to move to")]
    pub destination_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NowParams {
    #[schemars(description = "Timezone for the datetime: 'utc' or 'local' (default: 'local')")]
    pub timezone: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchReplaceEditParams {
    #[schemars(description = "The path to the file to edit")]
    pub path: String,
    #[schemars(description = "List of search and replace operations")]
    pub edits: Vec<EditOperation>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApplyPatchParams {
    #[schemars(description = "The path to the file to patch")]
    pub path: String,
    #[schemars(description = "The unified diff patch content")]
    pub patch: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskCompleteParams {
    #[schemars(description = "The ID of the task to mark as complete")]
    pub task_id: String,
    #[schemars(description = "The result or output of the task")]
    pub result: String,
}
