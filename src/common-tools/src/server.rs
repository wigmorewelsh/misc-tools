use crate::tools::CommonTools;
use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ListToolsResult, ProtocolVersion, ServerCapabilities,
    ServerCapabilitiesTools,
};
use rust_mcp_sdk::{
    mcp_server::{server_runtime, McpServerOptions, ServerHandler, ServerRuntime},
    schema::{
        schema_utils::CallToolError, CallToolRequestParams, CallToolResult, PaginatedRequestParams,
        RpcError,
    },
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
};
use std::sync::Arc;

pub struct CommonToolsHandler;

#[async_trait]
impl ServerHandler for CommonToolsHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: CommonTools::tools(),
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let tool_params: CommonTools = CommonTools::try_from(params).map_err(CallToolError::new)?;

        match tool_params {
            CommonTools::ReadFileTool(tool) => tool.call_tool().await,
            CommonTools::WriteFileTool(tool) => tool.call_tool().await,
            CommonTools::ExecuteCommandTool(tool) => tool.call_tool().await,
            CommonTools::ListDirectoryTool(tool) => tool.call_tool().await,
            CommonTools::CreateDirectoryTool(tool) => tool.call_tool().await,
            CommonTools::CopyPathTool(tool) => tool.call_tool().await,
            CommonTools::MovePathTool(tool) => tool.call_tool().await,
            CommonTools::NowTool(tool) => tool.call_tool().await,
            CommonTools::SearchReplaceEditTool(tool) => tool.call_tool().await,
            CommonTools::ApplyPatchTool(tool) => tool.call_tool().await,
            CommonTools::TaskCompleteTool(tool) => tool.call_tool().await,
        }
    }
}

pub fn create_server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "Common Tools MCP Server".into(),
            version: "0.1.0".into(),
            title: Some("Common Tools MCP Server".into()),
            description: Some("A collection of common file and system tools for MCP".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            resources: None,
            completions: None,
            tasks: None,
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Common tools for file operations, command execution, and system tasks".into(),
        ),
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    }
}

pub fn create_server() -> Result<Arc<ServerRuntime>, Box<dyn std::error::Error>> {
    let server_details = create_server_details();
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let handler = CommonToolsHandler {};

    let server = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
    });

    Ok(server)
}
