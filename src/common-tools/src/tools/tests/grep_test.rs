use crate::tools::GrepTool;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_grep_basic_search() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(
        temp_path.join("test1.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .await
    .unwrap();

    fs::write(
        temp_path.join("test2.py"),
        "def main():\n    print(\"Hello from Python\")\n",
    )
    .await
    .unwrap();

    let tool = GrepTool {
        regex: "Hello".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("test1.rs"));
    assert!(content.contains("test2.py"));
    assert!(content.contains("Hello, world!"));
    assert!(content.contains("Hello from Python"));
}

#[tokio::test]
async fn test_grep_case_sensitive() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(
        temp_path.join("case_test.txt"),
        "This file has UPPERCASE and lowercase text.\nUPPERCASE patterns should match.\n",
    )
    .await
    .unwrap();

    let tool_insensitive = GrepTool {
        regex: "uppercase".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool_insensitive.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);
    assert!(content.contains("UPPERCASE"));

    let tool_sensitive = GrepTool {
        regex: "uppercase".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: true,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool_sensitive.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);
    assert!(content.contains("No matches found"));
}

#[tokio::test]
async fn test_grep_include_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("src")).await.unwrap();
    fs::create_dir_all(temp_path.join("tests")).await.unwrap();

    fs::write(
        temp_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello from main\");\n}",
    )
    .await
    .unwrap();

    fs::write(
        temp_path.join("tests/test.rs"),
        "fn test_main() {\n    println!(\"Hello from test\");\n}",
    )
    .await
    .unwrap();

    fs::write(temp_path.join("README.md"), "# Project\nHello from README")
        .await
        .unwrap();

    let tool = GrepTool {
        regex: "Hello".to_string(),
        include_pattern: Some("**/*.rs".to_string()),
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("main.rs"));
    assert!(content.contains("test.rs"));
    assert!(!content.contains("README.md"));
}

#[tokio::test]
async fn test_grep_pagination() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    for i in 1..=25 {
        fs::write(
            temp_path.join(format!("file{}.txt", i)),
            format!("This is test file number {}\nwith search pattern\n", i),
        )
        .await
        .unwrap();
    }

    let tool = GrepTool {
        regex: "search pattern".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("use offset: 20"));
    assert!(content.contains("Results 1-20"));

    let tool_page2 = GrepTool {
        regex: "search pattern".to_string(),
        include_pattern: None,
        offset: 20,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result_page2 = tool_page2.call_tool().await.unwrap();
    let content_page2 = format!("{:?}", result_page2.content[0]);

    assert!(content_page2.contains("Found 25 total matches"));
}

#[tokio::test]
async fn test_grep_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(
        temp_path.join("test.txt"),
        "This file contains no special patterns",
    )
    .await
    .unwrap();

    let tool = GrepTool {
        regex: "nonexistent".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("No matches found"));
}

#[tokio::test]
async fn test_grep_invalid_regex() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let tool = GrepTool {
        regex: "[invalid".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_grep_context_lines() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(
        temp_path.join("context_test.txt"),
        "Line 1\nLine 2\nLine 3 with match\nLine 4\nLine 5\nLine 6\nLine 7\n",
    )
    .await
    .unwrap();

    let tool = GrepTool {
        regex: "with match".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3 with match"));
    assert!(content.contains("Line 4"));
    assert!(content.contains("Line 5"));
    assert!(content.contains("context_test.txt"));
    assert!(content.contains("L3 (context:"));
}

#[tokio::test]
async fn test_grep_respects_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("src")).await.unwrap();
    fs::create_dir_all(temp_path.join("target")).await.unwrap();

    fs::write(
        temp_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello from main\");\n}",
    )
    .await
    .unwrap();

    fs::write(
        temp_path.join("target/debug.log"),
        "DEBUG: Hello from debug log",
    )
    .await
    .unwrap();

    fs::write(temp_path.join("secret.txt"), "SECRET_KEY=hello_secret_data")
        .await
        .unwrap();

    fs::write(temp_path.join(".gitignore"), "target/\n*.log\nsecret.txt\n")
        .await
        .unwrap();

    let tool = GrepTool {
        regex: "Hello".to_string(),
        include_pattern: None,
        offset: 0,
        case_sensitive: false,
        working_directory: Some(temp_path.to_string_lossy().to_string()),
    };

    let result = tool.call_tool().await.unwrap();
    let content = format!("{:?}", result.content[0]);

    assert!(content.contains("main.rs"));
    assert!(content.contains("Hello from main"));
    assert!(!content.contains("debug.log"));
    assert!(!content.contains("secret.txt"));
    assert!(!content.contains("Hello from debug log"));
    assert!(!content.contains("hello_secret_data"));
}
