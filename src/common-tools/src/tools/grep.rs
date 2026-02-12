use ignore::WalkBuilder;
use regex::Regex;
use rmcp::model::{CallToolResult, Content};
use std::path::{Path, PathBuf};
use tokio::fs;

use super::{resolve_path, ToolError};

#[derive(Debug, Clone, Copy)]
struct LineNumber(usize);

impl LineNumber {
    fn value(self) -> usize {
        self.0
    }

    fn display(self) -> usize {
        self.0 + 1
    }
}

#[derive(Debug, Clone, Copy)]
struct FileSize(u64);

impl FileSize {
    const MAX_SIZE: FileSize = FileSize(10 * 1024 * 1024);

    fn exceeds_limit(self) -> bool {
        self.0 > Self::MAX_SIZE.0
    }
}

#[derive(Debug, Clone, Copy)]
struct ContextLines(usize);

impl ContextLines {
    const DEFAULT: ContextLines = ContextLines(2);

    fn start_from(self, line: LineNumber) -> usize {
        line.value().saturating_sub(self.0)
    }

    fn end_at(self, line: LineNumber, max_lines: usize) -> usize {
        std::cmp::min(line.value() + self.0, max_lines.saturating_sub(1))
    }
}

#[derive(Debug, Clone)]
struct SearchConfiguration {
    regex: Regex,
    file_filter: Option<glob::Pattern>,
    search_directory: PathBuf,
}

impl SearchConfiguration {
    fn new(tool: &GrepTool) -> Result<Self, ToolError> {
        let regex = Self::compile_regex(&tool.regex, tool.case_sensitive)?;
        let file_filter = Self::create_file_filter(&tool.include_pattern)?;
        let search_directory = Self::resolve_search_directory(&tool.working_directory);

        Ok(Self {
            regex,
            file_filter,
            search_directory,
        })
    }

    fn compile_regex(pattern: &str, case_sensitive: bool) -> Result<Regex, ToolError> {
        let regex_result = if case_sensitive {
            Regex::new(pattern)
        } else {
            regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
        };

        regex_result
            .map_err(|e| ToolError::InvalidArgument(format!("Regex compilation failed: {}", e)))
    }

    fn create_file_filter(pattern: &Option<String>) -> Result<Option<glob::Pattern>, ToolError> {
        match pattern {
            Some(pattern_str) => {
                let filter = glob::Pattern::new(pattern_str).map_err(|e| {
                    ToolError::InvalidArgument(format!("Invalid glob pattern: {}", e))
                })?;
                Ok(Some(filter))
            }
            None => Ok(None),
        }
    }

    fn resolve_search_directory(working_directory: &Option<String>) -> PathBuf {
        match working_directory {
            Some(dir) => resolve_path(dir, None),
            None => resolve_path(".", None),
        }
    }
}

pub struct GrepTool {
    pub regex: String,
    pub include_pattern: Option<String>,
    pub offset: u32,
    pub case_sensitive: bool,
    pub working_directory: Option<String>,
}

const MAX_RESULTS_PER_PAGE: u32 = 20;
const MAX_LINE_LENGTH: usize = 10_000;
const SAMPLE_SIZE_FOR_TEXT_DETECTION: usize = 8192;
const NON_TEXT_RATIO_THRESHOLD: f64 = 0.3;

impl GrepTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, ToolError> {
        let config = SearchConfiguration::new(self)?;
        let found_matches = self.search_files(&config).await;
        let output = self.format_results(&found_matches);

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    async fn search_files(&self, config: &SearchConfiguration) -> Vec<FileMatch> {
        let mut found_matches = Vec::new();
        let walker = self.create_file_walker(config);

        for dir_entry in walker.filter_map(|entry| entry.ok()) {
            if let Some(file_matches) = self.process_file(&dir_entry, config).await {
                found_matches.extend(file_matches);
            }
        }

        found_matches
    }

    fn create_file_walker(&self, config: &SearchConfiguration) -> ignore::Walk {
        WalkBuilder::new(&config.search_directory)
            .follow_links(false)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true)
            .parents(true)
            .require_git(false)
            .build()
    }

    async fn process_file(
        &self,
        dir_entry: &ignore::DirEntry,
        config: &SearchConfiguration,
    ) -> Option<Vec<FileMatch>> {
        if !self.should_search_file(dir_entry, config)? {
            return None;
        }

        let file_path = dir_entry.path();
        self.search_in_file(file_path, &config.regex).await.ok()
    }

    fn should_search_file(
        &self,
        dir_entry: &ignore::DirEntry,
        config: &SearchConfiguration,
    ) -> Option<bool> {
        if !dir_entry.file_type().is_some_and(|ft| ft.is_file()) {
            return Some(false);
        }

        let file_path = dir_entry.path();

        if !self.matches_file_filter(file_path, config) {
            return Some(false);
        }

        if !FileType::is_likely_text_file(file_path) {
            return Some(false);
        }

        Some(true)
    }

    fn matches_file_filter(&self, file_path: &Path, config: &SearchConfiguration) -> bool {
        match &config.file_filter {
            Some(filter) => {
                let relative_path = file_path
                    .strip_prefix(&config.search_directory)
                    .unwrap_or(file_path)
                    .to_string_lossy();
                filter.matches(&relative_path)
            }
            None => true,
        }
    }

    async fn search_in_file(
        &self,
        file_path: &Path,
        regex: &Regex,
    ) -> Result<Vec<FileMatch>, std::io::Error> {
        let file_content = self.read_file_safely(file_path).await?;

        if let Some(content) = file_content {
            Ok(self.find_matches_in_content(file_path, &content, regex))
        } else {
            Ok(Vec::new())
        }
    }

    async fn read_file_safely(&self, file_path: &Path) -> Result<Option<String>, std::io::Error> {
        if let Ok(metadata) = fs::metadata(file_path).await {
            if FileSize(metadata.len()).exceeds_limit() {
                return Ok(None);
            }
        }

        match fs::read_to_string(file_path).await {
            Ok(content) if FileType::is_likely_utf8_text(&content) => Ok(Some(content)),
            Ok(_) => Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn find_matches_in_content(
        &self,
        file_path: &Path,
        content: &str,
        regex: &Regex,
    ) -> Vec<FileMatch> {
        let content_lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (line_index, line_text) in content_lines.iter().enumerate() {
            if line_text.len() > MAX_LINE_LENGTH {
                continue;
            }

            if regex.is_match(line_text) {
                let match_line = LineNumber(line_index);
                let context = self.extract_context(&content_lines, match_line);

                matches.push(FileMatch::new(file_path.to_path_buf(), match_line, context));
            }
        }

        matches
    }

    fn extract_context(&self, content_lines: &[&str], match_line: LineNumber) -> ContextMatch {
        let context_lines = ContextLines::DEFAULT;
        let start_line = context_lines.start_from(match_line);
        let end_line = context_lines.end_at(match_line, content_lines.len());

        let context_snippet = content_lines[start_line..=end_line].join("\n");

        ContextMatch {
            start_line,
            end_line,
            content: context_snippet,
        }
    }

    fn format_results(&self, matches: &[FileMatch]) -> String {
        if matches.is_empty() {
            return "No matches found".to_string();
        }

        let paginated_matches = self.paginate_matches(matches);
        self.format_match_output(paginated_matches, matches.len() as u32)
    }

    fn paginate_matches<'a>(&self, matches: &'a [FileMatch]) -> &'a [FileMatch] {
        let start_index = self.offset as usize;
        let end_index = std::cmp::min(start_index + MAX_RESULTS_PER_PAGE as usize, matches.len());

        if start_index >= matches.len() {
            &[]
        } else {
            &matches[start_index..end_index]
        }
    }

    fn format_match_output(&self, matches: &[FileMatch], total_count: u32) -> String {
        let mut result = String::new();
        let mut current_file: Option<PathBuf> = None;

        for file_match in matches {
            if current_file.as_ref() != Some(&file_match.file_path) {
                result.push_str(&format!("\n## File: {}\n", file_match.file_path.display()));
                current_file = Some(file_match.file_path.clone());
            }

            result.push_str(&format!(
                "\n### L{} (context: {}-{})\n",
                file_match.match_line.display(),
                file_match.context.start_line + 1,
                file_match.context.end_line + 1
            ));
            result.push_str("```\n");
            result.push_str(&file_match.context.content);
            result.push_str("\n```\n");
        }

        self.add_pagination_info(result, matches.len() as u32, total_count)
    }

    fn add_pagination_info(&self, result: String, matches_shown: u32, total_count: u32) -> String {
        let has_more_results = (self.offset + matches_shown) < total_count;

        if has_more_results {
            format!(
                "Results {}-{} of {} total (use offset: {} for next page):{}\n",
                self.offset + 1,
                self.offset + matches_shown,
                total_count,
                self.offset + MAX_RESULTS_PER_PAGE,
                result
            )
        } else {
            format!("Found {} total matches:{}\n", total_count, result)
        }
    }
}

#[derive(Debug, Clone)]
struct ContextMatch {
    start_line: usize,
    end_line: usize,
    content: String,
}

#[derive(Debug, Clone)]
struct FileMatch {
    file_path: PathBuf,
    match_line: LineNumber,
    context: ContextMatch,
}

impl FileMatch {
    fn new(file_path: PathBuf, match_line: LineNumber, context: ContextMatch) -> Self {
        Self {
            file_path,
            match_line,
            context,
        }
    }
}

struct FileType;

impl FileType {
    const TEXT_EXTENSIONS: &'static [&'static str] = &[
        "txt", "rs", "py", "js", "ts", "jsx", "tsx", "html", "css", "scss", "json", "yaml", "yml",
        "toml", "md", "xml", "svg", "sh", "bash", "c", "cpp", "h", "hpp", "java", "kt", "go",
        "php", "rb", "lua", "sql", "conf", "log", "ini",
    ];

    const SPECIAL_FILES: &'static [&'static str] = &[
        "Dockerfile",
        "Makefile",
        "Rakefile",
        "Gemfile",
        "README",
        "LICENSE",
        ".gitignore",
        ".env",
    ];

    fn is_likely_text_file(path: &Path) -> bool {
        Self::has_text_extension(path) || Self::is_special_file(path)
    }

    fn has_text_extension(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::TEXT_EXTENSIONS.contains(&ext)
        } else {
            false
        }
    }

    fn is_special_file(path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            Self::SPECIAL_FILES.contains(&name)
        } else {
            false
        }
    }

    fn is_likely_utf8_text(content: &str) -> bool {
        if content.is_empty() {
            return true;
        }

        let sample_size = std::cmp::min(content.len(), SAMPLE_SIZE_FOR_TEXT_DETECTION);
        let sample = &content[..sample_size];

        let null_bytes = sample.bytes().filter(|&b| b == 0).count();
        let non_printable = sample
            .chars()
            .filter(|&c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            .count();

        let total_chars = sample.chars().count();
        if total_chars == 0 {
            return false;
        }

        let non_text_ratio = (null_bytes + non_printable) as f64 / total_chars as f64;
        non_text_ratio < NON_TEXT_RATIO_THRESHOLD
    }
}
