use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::{resolve_path, ToolError};

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct EditOperation {
    /// The text to search for. Can include multiple lines and doesn't need exact whitespace matching.
    pub search: String,
    /// The text to replace it with. Use empty string to delete.
    pub replace: String,
}

pub struct SearchReplaceEditTool {
    pub path: String,
    pub edits: Vec<EditOperation>,
}

impl SearchReplaceEditTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let abs_path = resolve_path(&self.path, None);

        if !abs_path.exists() {
            return Err(ToolError::FileNotFound(abs_path.display().to_string()).into());
        }

        let original_content = fs::read_to_string(&abs_path).await.map_err(ToolError::Io)?;
        let result_content = apply_all_edits(&original_content, &self.edits)?;

        fs::write(&abs_path, &result_content)
            .await
            .map_err(ToolError::Io)?;

        let message = format!(
            "Successfully applied {} edits to {}",
            self.edits.len(),
            abs_path.display()
        );
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}

fn apply_all_edits(content: &str, edits: &[EditOperation]) -> Result<String, ToolError> {
    let mut result_content = content.to_string();

    for (i, edit) in edits.iter().enumerate() {
        result_content = apply_edit(&result_content, edit)
            .map_err(|e| ToolError::Other(format!("Edit {} failed: {}", i + 1, e)))?;
    }

    Ok(result_content)
}

fn apply_edit(content: &str, edit: &EditOperation) -> Result<String, String> {
    if content.contains(&edit.search) {
        let new_content = content.replacen(&edit.search, &edit.replace, 1);
        return Ok(new_content);
    }

    fuzzy_search_replace(content, &edit.search, &edit.replace)
}

fn fuzzy_search_replace(content: &str, search: &str, replace: &str) -> Result<String, String> {
    let search_normalized = normalize_whitespace(search);
    let search_lines: Vec<&str> = search_normalized.split('\n').collect();
    let content_lines: Vec<&str> = content.split('\n').collect();

    let mut matches = Vec::new();

    for start_idx in 0..=content_lines.len().saturating_sub(search_lines.len()) {
        let mut match_candidate = true;
        let match_end_idx = start_idx + search_lines.len();

        for (i, search_line) in search_lines.iter().enumerate() {
            if start_idx + i >= content_lines.len() {
                match_candidate = false;
                break;
            }

            let content_line = content_lines[start_idx + i];
            let content_normalized = normalize_whitespace(content_line);

            if content_normalized != *search_line {
                match_candidate = false;
                break;
            }
        }

        if match_candidate {
            matches.push((start_idx, match_end_idx));
        }
    }

    if matches.is_empty() {
        return Err(
            "Search text not found in file. Please verify the text exists and check for typos."
                .to_string(),
        );
    }

    if matches.len() > 1 {
        return Err(format!(
            "Ambiguous: found {} potential matches",
            matches.len()
        ));
    }

    let (start_idx, end_idx) = matches[0];
    let first_content_line = content_lines[start_idx];
    let indentation = extract_indentation(first_content_line);

    let replace_lines: Vec<&str> = replace.split('\n').collect();
    let mut indented_replace_lines = Vec::new();

    if !replace_lines.is_empty() {
        indented_replace_lines.push(format!("{}{}", indentation, replace_lines[0]));
        for line in replace_lines.iter().skip(1) {
            if line.trim().is_empty() {
                indented_replace_lines.push(line.to_string());
            } else {
                indented_replace_lines.push(format!("{}{}", indentation, line));
            }
        }
    }

    let mut new_content_lines = Vec::new();
    new_content_lines.extend_from_slice(&content_lines[..start_idx]);
    new_content_lines.extend(indented_replace_lines.iter().map(|s| s.as_str()));
    new_content_lines.extend_from_slice(&content_lines[end_idx..]);

    Ok(new_content_lines.join("\n"))
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_indentation(line: &str) -> String {
    line.chars().take_while(|c| c.is_whitespace()).collect()
}
