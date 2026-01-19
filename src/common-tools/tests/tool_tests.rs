use common_tools::tools::*;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_read_file_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Hello, world!";

    fs::write(&file_path, content).unwrap();

    let tool = ReadFileTool {
        path: file_path.to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Hello, world!"));
}

#[tokio::test]
async fn test_write_file_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("output.txt");
    let content = "Test content";

    let tool = WriteFileTool {
        path: file_path.to_string_lossy().to_string(),
        content: content.to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully wrote to"));

    let written_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(written_content, content);
}

#[tokio::test]
async fn test_list_directory_tool() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let tool = ListDirectoryTool {
        path: temp_dir.path().to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("file1.txt"));
    assert!(content_str.contains("file2.txt"));
    assert!(content_str.contains("subdir"));
    assert!(content_str.contains("3 entries"));
}

#[tokio::test]
async fn test_create_directory_tool() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir_path = temp_dir.path().join("new_directory");

    let tool = CreateDirectoryTool {
        path: new_dir_path.to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully created directory"));
    assert!(new_dir_path.exists());
    assert!(new_dir_path.is_dir());
}

#[tokio::test]
async fn test_copy_path_tool() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("source.txt");
    let dest_file = temp_dir.path().join("dest.txt");
    let content = "Copy test content";

    fs::write(&source_file, content).unwrap();

    let tool = CopyPathTool {
        source_path: source_file.to_string_lossy().to_string(),
        destination_path: dest_file.to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully copied file to"));
    assert!(dest_file.exists());

    let copied_content = fs::read_to_string(&dest_file).unwrap();
    assert_eq!(copied_content, content);
}

#[tokio::test]
async fn test_move_path_tool() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("source.txt");
    let dest_file = temp_dir.path().join("dest.txt");
    let content = "Move test content";

    fs::write(&source_file, content).unwrap();

    let tool = MovePathTool {
        source_path: source_file.to_string_lossy().to_string(),
        destination_path: dest_file.to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully moved to"));
    assert!(!source_file.exists());
    assert!(dest_file.exists());

    let moved_content = fs::read_to_string(&dest_file).unwrap();
    assert_eq!(moved_content, content);
}

#[tokio::test]
async fn test_now_tool() {
    let tool = NowTool {
        timezone: "utc".to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Current time (utc)"));
    assert!(content_str.contains("T"));
    // UTC time should contain either Z suffix or +00:00
    assert!(content_str.contains("Z") || content_str.contains("+00:00"));
}

#[tokio::test]
async fn test_now_tool_local() {
    let tool = NowTool {
        timezone: "local".to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Current time (local)"));
    assert!(content_str.contains("T"));
}

#[tokio::test]
async fn test_execute_command_tool() {
    let tool = ExecuteCommandTool {
        command: if cfg!(target_os = "windows") {
            "echo Hello".to_string()
        } else {
            "echo 'Hello'".to_string()
        },
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Hello"));
}

#[tokio::test]
async fn test_search_replace_edit_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("edit_test.txt");
    let original_content = "Hello world\nThis is a test\nGoodbye world";

    fs::write(&file_path, original_content).unwrap();

    let tool = SearchReplaceEditTool {
        path: file_path.to_string_lossy().to_string(),
        edits: vec![EditOperation {
            search: "Hello".to_string(),
            replace: "Hi".to_string(),
        }],
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully applied"));
    assert!(content_str.contains("1 edits"));

    let edited_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(edited_content, "Hi world\nThis is a test\nGoodbye world");
}

#[tokio::test]
async fn test_task_complete_tool() {
    let tool = TaskCompleteTool {
        summary: "Test task completed successfully".to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Test task completed successfully"));
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let tool = ReadFileTool {
        path: "/nonexistent/file.txt".to_string(),
    };

    let result = tool.call_tool().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_nonexistent_directory() {
    let tool = ListDirectoryTool {
        path: "/nonexistent/directory".to_string(),
    };

    let result = tool.call_tool().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_copy_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let dest_file = temp_dir.path().join("dest.txt");

    let tool = CopyPathTool {
        source_path: "/nonexistent/file.txt".to_string(),
        destination_path: dest_file.to_string_lossy().to_string(),
    };

    let result = tool.call_tool().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_failing_command() {
    let tool = ExecuteCommandTool {
        command: "nonexistent_command_xyz123".to_string(),
    };

    let result = tool.call_tool().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_apply_patch_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("patch_test.txt");
    let original_content = "line1\nline2\nline3";

    fs::write(&file_path, original_content).unwrap();

    let patch = "@@ -1,3 +1,3 @@\n line1\n-line2\n+modified line2\n line3";

    let tool = ApplyPatchTool {
        path: file_path.to_string_lossy().to_string(),
        patch: patch.to_string(),
    };

    let result = tool.call_tool().await.unwrap();

    assert_eq!(result.content.len(), 1);
    let content_str = format!("{:?}", result.content[0]);
    assert!(content_str.contains("Successfully applied patch"));

    let patched_content = fs::read_to_string(&file_path).unwrap();
    assert!(patched_content.contains("modified line2"));
}
