use common_tools::CommonToolsServer;
use rmcp::{
    model::{CallToolRequestParams, ClientRequest, Request, ServerResult},
    ClientHandler, ServiceExt,
};
use serde_json::json;
use tempfile::TempDir;
use tokio::fs;

#[derive(Default, Clone)]
struct TestClient;

impl ClientHandler for TestClient {}

#[tokio::test]
async fn test_server_list_tools() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let response = client_service
        .send_request(ClientRequest::ListToolsRequest(Default::default()))
        .await?;

    let ServerResult::ListToolsResult(result) = response else {
        panic!("expected list tools result, got {response:?}");
    };

    assert_eq!(result.tools.len(), 12);

    let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(tool_names.contains(&"read_file"));
    assert!(tool_names.contains(&"write_file"));
    assert!(tool_names.contains(&"grep"));
    assert!(tool_names.contains(&"execute_command"));
    assert!(tool_names.contains(&"list_directory"));
    assert!(tool_names.contains(&"create_directory"));
    assert!(tool_names.contains(&"copy_path"));
    assert!(tool_names.contains(&"move_path"));
    assert!(tool_names.contains(&"now"));
    assert!(tool_names.contains(&"search_replace_edit"));
    assert!(tool_names.contains(&"apply_patch"));
    assert!(tool_names.contains(&"task_complete"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_read_file_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Integration test content";

    fs::write(&file_path, content).await.unwrap();

    let params = CallToolRequestParams {
        name: "read_file".into(),
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

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Integration test content"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_write_file_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("output.txt");
    let content = "Integration test output";

    let params = CallToolRequestParams {
        name: "write_file".into(),
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

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully wrote to"));
    assert!(file_path.exists());

    let written_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(written_content, content);

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_execute_command_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let command = if cfg!(target_os = "windows") {
        "echo Integration Test"
    } else {
        "echo 'Integration Test'"
    };

    let params = CallToolRequestParams {
        name: "execute_command".into(),
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

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Integration Test"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_list_directory_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("file1.txt"), "content1")
        .await
        .unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2")
        .await
        .unwrap();

    let params = CallToolRequestParams {
        name: "list_directory".into(),
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

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("file1.txt"));
    assert!(content_str.contains("file2.txt"));
    assert!(content_str.contains("2 entries"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_now_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let params = CallToolRequestParams {
        name: "now".into(),
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

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Current time (utc)"));
    assert!(content_str.contains("T"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_call_task_complete_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let params = CallToolRequestParams {
        name: "task_complete".into(),
        arguments: Some(
            json!({
                "task_id": "test-task-123",
                "result": "Integration test completed successfully"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        meta: None,
        task: None,
    };

    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await?;

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected call tool result, got {response:?}");
    };

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("test-task-123"));
    assert!(content_str.contains("Integration test completed successfully"));

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_invalid_tool() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let params = CallToolRequestParams {
        name: "nonexistent_tool".into(),
        arguments: Some(json!({}).as_object().unwrap().clone()),
        meta: None,
        task: None,
    };

    let result = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await;

    assert!(result.is_err());

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_server_tool_with_invalid_arguments() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let params = CallToolRequestParams {
        name: "read_file".into(),
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

    let result = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await;

    assert!(result.is_err());

    client_service.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_integration_file_workflow() -> anyhow::Result<()> {
    let server = CommonToolsServer::new();
    let client = TestClient;
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client_service = client.serve(client_transport).await?;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("workflow_test.txt");
    let original_content = "Original content for workflow test";
    let new_content = "Modified content for workflow test";

    let write_params = CallToolRequestParams {
        name: "write_file".into(),
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

    let write_response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(write_params)))
        .await?;

    let ServerResult::CallToolResult(write_result) = write_response else {
        panic!("expected call tool result");
    };

    assert_eq!(write_result.content.len(), 1);
    let write_content_str = format!("{:?}", write_result.content[0]);
    assert!(write_content_str.contains("Successfully wrote"));

    let read_params = CallToolRequestParams {
        name: "read_file".into(),
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

    let read_response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(read_params)))
        .await?;

    let ServerResult::CallToolResult(read_result) = read_response else {
        panic!("expected call tool result");
    };

    assert_eq!(read_result.content.len(), 1);
    let read_content_str = format!("{:?}", read_result.content[0]);
    assert!(read_content_str.contains("Original content for workflow test"));

    let edit_params = CallToolRequestParams {
        name: "search_replace_edit".into(),
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

    let edit_response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(edit_params)))
        .await?;

    let ServerResult::CallToolResult(edit_result) = edit_response else {
        panic!("expected call tool result");
    };

    assert_eq!(edit_result.content.len(), 1);
    let edit_content_str = format!("{:?}", edit_result.content[0]);
    assert!(edit_content_str.contains("Successfully applied"));
    assert!(edit_content_str.contains("1 edits"));

    let final_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(final_content, new_content);

    let list_params = CallToolRequestParams {
        name: "list_directory".into(),
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

    let list_response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(list_params)))
        .await?;

    let ServerResult::CallToolResult(list_result) = list_response else {
        panic!("expected call tool result");
    };

    assert_eq!(list_result.content.len(), 1);
    let list_content_str = format!("{:?}", list_result.content[0]);
    assert!(list_content_str.contains("workflow_test.txt"));
    assert!(list_content_str.contains("1 entries"));

    client_service.cancel().await?;
    Ok(())
}
