use crate::config::Config;
use crate::markdown_parser::MarkdownParser;
use crate::mermaid_parser::MermaidParser;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LintIssue {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: crate::config::Severity,
    pub rule: String,
    pub fix: Option<String>, // TODO: proper fix struct
}

pub struct Linter {
    config: Config,
    markdown_parser: MarkdownParser,
    mermaid_parser: MermaidParser,
}

impl Linter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            markdown_parser: MarkdownParser::new(),
            mermaid_parser: MermaidParser::new(),
        }
    }

    pub fn lint_path(&self, path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        if path.is_file() {
            issues.extend(self.lint_file(path)?);
        } else if path.is_dir() {
            issues.extend(self.lint_directory(path)?);
        } else {
            return Err(anyhow::anyhow!("Path does not exist"));
        }

        Ok(issues)
    }

    pub fn lint_content(&self, content: &str) -> anyhow::Result<Vec<LintIssue>> {
        let mermaid_blocks = self.markdown_parser.extract_mermaid_blocks(content);

        let mut issues = Vec::new();
        for (line_start, code) in mermaid_blocks {
            let ast = self.mermaid_parser.parse(&code)?;
            issues.extend(self.mermaid_parser.validate(&ast, &self.config).into_iter().map(|mut issue| {
                issue.line += line_start;
                issue
            }));
        }

        Ok(issues)
    }

    fn lint_file(&self, path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let content = fs::read_to_string(path)?;
        let mermaid_blocks = self.markdown_parser.extract_mermaid_blocks(&content);

        let mut issues = Vec::new();
        for (line_start, code) in mermaid_blocks {
            let ast = self.mermaid_parser.parse(&code)?;
            issues.extend(self.mermaid_parser.validate(&ast, &self.config).into_iter().map(|mut issue| {
                issue.file = path.to_path_buf();
                issue.line += line_start;
                issue
            }));
        }

        Ok(issues)
    }

    fn lint_directory(&self, path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        use walkdir::WalkDir;
        use rayon::prelude::*;

        let files: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            .collect();

        let all_issues: Vec<LintIssue> = files.par_iter()
            .filter_map(|entry| self.lint_file(entry.path()).ok())
            .flatten()
            .collect();
        Ok(all_issues)
    }
}