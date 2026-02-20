use rmcp::model::{CallToolResult, Content};
use tokio::fs;

use super::{resolve_path, ToolError};

pub struct ApplyPatchTool {
    pub path: String,
    pub patch: String,
}

impl ApplyPatchTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let abs_path = resolve_path(&self.path, None);

        if !abs_path.exists() {
            return Err(ToolError::FileNotFound(abs_path.display().to_string()));
        }

        let original_content = fs::read_to_string(&abs_path).await.map_err(ToolError::Io)?;
        let original_lines: Vec<&str> = original_content.lines().collect();

        let normalized_patch = self.patch.replace("\r\n", "\n").replace('\r', "\n");

        let patched_lines =
            apply_unified_diff(&original_lines, &normalized_patch).ok_or_else(|| {
                ToolError::Other("Failed to apply patch - hunks did not match".to_string())
            })?;

        let patched_content = patched_lines.join("\n");
        let final_content = if original_content.ends_with('\n') {
            format!("{}\n", patched_content)
        } else {
            patched_content
        };

        fs::write(&abs_path, &final_content)
            .await
            .map_err(ToolError::Io)?;

        let message = format!("Successfully applied patch to {}", abs_path.display());
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}

#[allow(dead_code)]
struct HunkData {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    context_and_removed: Vec<String>,
    new_lines: Vec<String>,
}

fn apply_unified_diff(original_lines: &[&str], patch: &str) -> Option<Vec<String>> {
    let patch_lines: Vec<&str> = patch.lines().collect();
    if patch_lines.is_empty() {
        return None;
    }

    let hunks = parse_unified_diff(&patch_lines)?;
    if hunks.is_empty() {
        return None;
    }

    let mut result: Vec<String> = original_lines.iter().map(|s| s.to_string()).collect();

    for hunk in hunks.iter().rev() {
        if hunk.old_start == 0 || hunk.old_start > result.len() + 1 {
            return None;
        }

        let idx = hunk.old_start - 1;
        if idx + hunk.old_count > result.len() {
            return None;
        }

        let actual_lines: Vec<&str> = result[idx..idx + hunk.old_count]
            .iter()
            .map(|s| s.as_str())
            .collect();

        if actual_lines.len() != hunk.context_and_removed.len() {
            return None;
        }

        for (actual, expected) in actual_lines.iter().zip(hunk.context_and_removed.iter()) {
            let actual_normalized = actual.split_whitespace().collect::<Vec<_>>().join(" ");
            let expected_normalized = expected.split_whitespace().collect::<Vec<_>>().join(" ");

            if actual_normalized != expected_normalized {
                return None;
            }
        }

        result.splice(idx..idx + hunk.old_count, hunk.new_lines.iter().cloned());
    }

    Some(result)
}

fn parse_unified_diff(patch_lines: &[&str]) -> Option<Vec<HunkData>> {
    let mut hunks = Vec::new();
    let mut i = 0;

    while i < patch_lines.len() {
        let line = patch_lines[i];

        if line.starts_with("@@") {
            let parts: Vec<&str> = line.split("@@").collect();
            if parts.len() < 2 {
                i += 1;
                continue;
            }

            let ranges: Vec<&str> = parts[1].split_whitespace().collect();
            if ranges.len() < 2 {
                i += 1;
                continue;
            }

            let old_range: Vec<&str> = ranges[0].trim_start_matches('-').split(',').collect();
            let new_range: Vec<&str> = ranges[1].trim_start_matches('+').split(',').collect();

            let old_start = old_range[0].parse::<usize>().ok()?;
            let old_count = if old_range.len() > 1 {
                old_range[1].parse::<usize>().ok()?
            } else {
                1
            };

            let new_start = new_range[0].parse::<usize>().ok()?;
            let new_count = if new_range.len() > 1 {
                new_range[1].parse::<usize>().ok()?
            } else {
                1
            };

            i += 1;
            let mut context_and_removed = Vec::new();
            let mut new_lines = Vec::new();

            while i < patch_lines.len() && !patch_lines[i].starts_with("@@") {
                let hunk_line = patch_lines[i];
                if let Some(content) = hunk_line.strip_prefix(' ') {
                    context_and_removed.push(content.to_string());
                    new_lines.push(content.to_string());
                } else if let Some(content) = hunk_line.strip_prefix('-') {
                    context_and_removed.push(content.to_string());
                } else if let Some(content) = hunk_line.strip_prefix('+') {
                    new_lines.push(content.to_string());
                } else if hunk_line.starts_with('\\') {
                    // "No newline at end of file" marker - skip
                } else if hunk_line.trim().is_empty() {
                    context_and_removed.push(String::new());
                    new_lines.push(String::new());
                } else {
                    break;
                }
                i += 1;
            }

            hunks.push(HunkData {
                old_start,
                old_count,
                new_start,
                new_count,
                context_and_removed,
                new_lines,
            });
        } else {
            i += 1;
        }
    }

    Some(hunks)
}
