use crate::tools::*;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};

#[derive(Clone)]
pub struct CommonToolsServer {
    tool_router: ToolRouter<Self>,
}

impl Default for CommonToolsServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl CommonToolsServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Read the contents of a text file from the file system")]
    async fn read_file(
        &self,
        Parameters(params): Parameters<ReadFileParams>,
    ) -> Result<CallToolResult, McpError> {
        ReadFileTool { path: params.path }
            .call_tool()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Write content to a file, creating it if it doesn't exist")]
    async fn write_file(
        &self,
        Parameters(params): Parameters<WriteFileParams>,
    ) -> Result<CallToolResult, McpError> {
        WriteFileTool {
            path: params.path,
            content: params.content,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Execute a shell command in a terminal and return the output")]
    async fn execute_command(
        &self,
        Parameters(params): Parameters<ExecuteCommandParams>,
    ) -> Result<CallToolResult, McpError> {
        ExecuteCommandTool {
            command: params.command,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Search file contents using regular expressions")]
    async fn grep(
        &self,
        Parameters(params): Parameters<GrepParams>,
    ) -> Result<CallToolResult, McpError> {
        GrepTool {
            regex: params.regex,
            include_pattern: params.include_pattern,
            offset: params.offset.unwrap_or(0),
            case_sensitive: params.case_sensitive.unwrap_or(false),
            working_directory: params.working_directory,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "List files and directories in a given path")]
    async fn list_directory(
        &self,
        Parameters(params): Parameters<ListDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        ListDirectoryTool { path: params.path }
            .call_tool()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Create a new directory at the specified path")]
    async fn create_directory(
        &self,
        Parameters(params): Parameters<CreateDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        CreateDirectoryTool { path: params.path }
            .call_tool()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Copy a file or directory to a new location")]
    async fn copy_path(
        &self,
        Parameters(params): Parameters<CopyPathParams>,
    ) -> Result<CallToolResult, McpError> {
        CopyPathTool {
            source_path: params.source_path,
            destination_path: params.destination_path,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Move or rename a file or directory")]
    async fn move_path(
        &self,
        Parameters(params): Parameters<MovePathParams>,
    ) -> Result<CallToolResult, McpError> {
        MovePathTool {
            source_path: params.source_path,
            destination_path: params.destination_path,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get the current date and time in ISO 8601 format")]
    async fn now(
        &self,
        Parameters(params): Parameters<NowParams>,
    ) -> Result<CallToolResult, McpError> {
        NowTool {
            timezone: params.timezone.unwrap_or_else(|| "local".to_string()),
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Apply search and replace edits to a file")]
    async fn search_replace_edit(
        &self,
        Parameters(params): Parameters<SearchReplaceEditParams>,
    ) -> Result<CallToolResult, McpError> {
        SearchReplaceEditTool {
            path: params.path,
            edits: params.edits,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Apply a unified diff patch to a file")]
    async fn apply_patch(
        &self,
        Parameters(params): Parameters<ApplyPatchParams>,
    ) -> Result<CallToolResult, McpError> {
        ApplyPatchTool {
            path: params.path,
            patch: params.patch,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Mark a task as complete")]
    async fn task_complete(
        &self,
        Parameters(params): Parameters<TaskCompleteParams>,
    ) -> Result<CallToolResult, McpError> {
        TaskCompleteTool {
            task_id: params.task_id,
            result: params.result,
        }
        .call_tool()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
}

#[tool_handler]
impl ServerHandler for CommonToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Common Tools MCP Server".into(),
                version: "0.1.0".into(),
                title: None,
                description: Some("A collection of common file and system tools for MCP".into()),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Common tools for file operations, command execution, and system tasks".into(),
            ),
        }
    }
}
