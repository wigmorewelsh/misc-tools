use common_tools::{create_server_details, CommonToolsHandler};
use rust_mcp_sdk::schema::CallToolRequestParams;
use rust_mcp_sdk::{
    mcp_server::{server_runtime, McpServerOptions, ServerHandler, ServerRuntime},
    StdioTransport, ToMcpServerHandler, TransportOptions,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

async fn create_test_server() -> Arc<ServerRuntime> {
    let server_details = create_server_details();
    let transport = StdioTransport::new(TransportOptions::default()).unwrap();
    let handler = CommonToolsHandler {};

    server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
    })
}

#[tokio::test]
async fn test_server_list_tools() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let result = handler
        .handle_list_tools_request(None, server)
        .await
        .unwrap();

    assert_eq!(result.tools.len(), 11);

    let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"read_file"));
    assert!(tool_names.contains(&"write_file"));
    assert!(tool_names.contains(&"execute_command"));
    assert!(tool_names.contains(&"list_directory"));
    assert!(tool_names.contains(&"create_directory"));
    assert!(tool_names.contains(&"copy_path"));
    assert!(tool_names.contains(&"move_path"));
    assert!(tool_names.contains(&"now"));
    assert!(tool_names.contains(&"search_replace_edit"));
    assert!(tool_names.contains(&"apply_patch"));
    assert!(tool_names.contains(&"task_complete"));
}

#[tokio::test]
async fn test_server_call_read_file_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Integration test content";

    fs::write(&file_path, content).await.unwrap();

    let params = CallToolRequestParams {
        name: "read_file".to_string(),
        arguments: Some(
            json!({
                "path": file_path.to_string_lossy()
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Integration test content"));
}

#[tokio::test]
async fn test_server_call_write_file_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("output.txt");
    let content = "Integration test output";

    let params = CallToolRequestParams {
        name: "write_file".to_string(),
        arguments: Some(
            json!({
                "path": file_path.to_string_lossy(),
                "content": content
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully wrote to"));
    assert!(file_path.exists());

    let written_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(written_content, content);
}

#[tokio::test]
async fn test_server_call_execute_command_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let command = if cfg!(target_os = "windows") {
        "echo Integration Test"
    } else {
        "echo 'Integration Test'"
    };

    let params = CallToolRequestParams {
        name: "execute_command".to_string(),
        arguments: Some(
            json!({
                "command": command
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Integration Test"));
}

#[tokio::test]
async fn test_server_call_list_directory_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("file1.txt"), "content1")
        .await
        .unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2")
        .await
        .unwrap();

    let params = CallToolRequestParams {
        name: "list_directory".to_string(),
        arguments: Some(
            json!({
                "path": temp_dir.path().to_string_lossy()
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("file1.txt"));
    assert!(content_str.contains("file2.txt"));
    assert!(content_str.contains("2 entries"));
}

#[tokio::test]
async fn test_server_call_now_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let params = CallToolRequestParams {
        name: "now".to_string(),
        arguments: Some(
            json!({
                "timezone": "utc"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Current time (utc)"));
    assert!(content_str.contains("T"));
}

#[tokio::test]
async fn test_server_call_task_complete_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let params = CallToolRequestParams {
        name: "task_complete".to_string(),
        arguments: Some(
            json!({
                "summary": "Integration test completed successfully"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler
        .handle_call_tool_request(params, server)
        .await
        .unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Integration test completed successfully"));
}

#[tokio::test]
async fn test_server_invalid_tool() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let params = CallToolRequestParams {
        name: "nonexistent_tool".to_string(),
        arguments: Some(json!({}).as_object().unwrap().clone()),
        meta: None,
        task: None,
    };

    let result = handler.handle_call_tool_request(params, server).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_server_tool_with_invalid_arguments() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};

    let params = CallToolRequestParams {
        name: "read_file".to_string(),
        arguments: Some(
            json!({
                "invalid_field": "some_value"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let result = handler.handle_call_tool_request(params, server).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_integration_file_workflow() {
    let server = create_test_server().await;
    let handler = CommonToolsHandler {};
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("workflow_test.txt");
    let original_content = "Original content for workflow test";
    let new_content = "Modified content for workflow test";

    // Step 1: Write a file
    let write_params = CallToolRequestParams {
        name: "write_file".to_string(),
        arguments: Some(
            json!({
                "path": file_path.to_string_lossy(),
                "content": original_content
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let write_result = handler
        .handle_call_tool_request(write_params, server.clone())
        .await
        .unwrap();
    assert_eq!(write_result.content.len(), 1);
    let write_content_str = format!("{:?}", write_result.content[0]);
    assert!(write_content_str.contains("Successfully wrote"));

    // Step 2: Read the file back
    let read_params = CallToolRequestParams {
        name: "read_file".to_string(),
        arguments: Some(
            json!({
                "path": file_path.to_string_lossy()
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let read_result = handler
        .handle_call_tool_request(read_params, server.clone())
        .await
        .unwrap();
    assert_eq!(read_result.content.len(), 1);
    let read_content_str = format!("{:?}", read_result.content[0]);
    assert!(read_content_str.contains("Original content for workflow test"));

    // Step 3: Search and replace in the file
    let edit_params = CallToolRequestParams {
        name: "search_replace_edit".to_string(),
        arguments: Some(
            json!({
                "path": file_path.to_string_lossy(),
                "edits": [{
                    "search": "Original",
                    "replace": "Modified"
                }]
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let edit_result = handler
        .handle_call_tool_request(edit_params, server.clone())
        .await
        .unwrap();
    assert_eq!(edit_result.content.len(), 1);
    let edit_content_str = format!("{:?}", edit_result.content[0]);
    assert!(edit_content_str.contains("Successfully applied"));
    assert!(edit_content_str.contains("1 edits"));

    // Step 4: Verify the changes
    let final_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(final_content, new_content);

    // Step 5: List the directory to confirm file exists
    let list_params = CallToolRequestParams {
        name: "list_directory".to_string(),
        arguments: Some(
            json!({
                "path": temp_dir.path().to_string_lossy()
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let list_result = handler
        .handle_call_tool_request(list_params, server)
        .await
        .unwrap();
    assert_eq!(list_result.content.len(), 1);
    let list_content_str = format!("{:?}", list_result.content[0]);
    assert!(list_content_str.contains("workflow_test.txt"));
    assert!(list_content_str.contains("1 entries"));
}
