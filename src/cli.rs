use crate::linter::Linter;
use crate::config::Config;
// use crate::lsp_server::LSPServer;
use std::path::Path;

pub async fn run_lint(path: &str, format: &str, fix: bool) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let config = Config::load(None)?;
    let linter = Linter::new(config);
    let issues = linter.lint_path(Path::new(path))?;
    let duration = start.elapsed();

    match format {
        "text" => print_issues_text(&issues),
        "json" => print_issues_json(&issues)?,
        "sarif" => print_issues_sarif(&issues)?,
        _ => println!("Unknown format: {}", format),
    }

    eprintln!("Linting completed in {:.2}s", duration.as_secs_f64());

    if fix {
        apply_fixes(&issues, Path::new(path))?;
    }

    Ok(())
}

// pub async fn run_lsp() -> anyhow::Result<()> {
//     LSPServer::run().await?;
//     Ok(())
// }

fn print_issues_text(issues: &[crate::linter::LintIssue]) {
    for issue in issues {
        println!(
            "{}:{}:{} {} [{}] {}",
            issue.file.display(),
            issue.line,
            issue.column,
            issue.severity,
            issue.rule,
            issue.message
        );
    }
}

fn print_issues_json(issues: &[crate::linter::LintIssue]) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(issues)?);
    Ok(())
}

fn print_issues_sarif(issues: &[crate::linter::LintIssue]) -> anyhow::Result<()> {
    let mut rules_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    for i in issues {
        rules_map.entry(i.rule.clone()).or_insert_with(|| serde_json::json!({
            "id": i.rule,
            "name": i.rule,
            "shortDescription": {"text": i.message}
        }));
    }

    let results: Vec<serde_json::Value> = issues.iter().map(|i| serde_json::json!({
        "ruleId": i.rule,
        "level": match i.severity {
            crate::config::Severity::Error => "error",
            crate::config::Severity::Warning => "warning",
            crate::config::Severity::Info => "note",
        },
        "message": {"text": i.message},
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {"uri": i.file.display().to_string()},
                "region": {
                    "startLine": i.line + 1,
                    "startColumn": i.column + 1
                }
            }
        }]
    })).collect();

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "meerjungfraulint",
                    "rules": rules_map.values().collect::<Vec<_>>()
                }
            },
            "results": results
        }]
    });

    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}

fn apply_fixes(issues: &[crate::linter::LintIssue], path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    for issue in issues {
        if issue.rule == "naming-convention" {
            if let Some(fix) = &issue.fix {
                if let Some(line) = lines.get(issue.line as usize) {
                    if line.contains("class ") {
                        // Extract old name from message: "Node 'old' should be PascalCase"
                        if let Some(old_name) = issue.message.split("'").nth(1) {
                            let new_line = line.replace(&format!("class {}", old_name), &format!("class {}", fix));
                            new_lines[issue.line as usize] = new_line;
                        }
                    }
                }
            }
        }
    }

    let new_content = new_lines.join("\n");
    std::fs::write(path, new_content)?;
    println!("Applied fixes to {}", path.display());
    Ok(())
}