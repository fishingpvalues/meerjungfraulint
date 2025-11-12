use crate::config::Config;
use crate::linter::LintIssue;


#[derive(Debug)]
pub struct MermaidAST {
    pub diagram_type: DiagramType,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub enum DiagramType {
    ClassDiagram,
    Flowchart,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

pub struct MermaidParser;

impl MermaidParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, code: &str) -> anyhow::Result<MermaidAST> {
        // Simple stub parser
        let diagram_type = if code.contains("classDiagram") {
            DiagramType::ClassDiagram
        } else if code.contains("flowchart") {
            DiagramType::Flowchart
        } else {
            DiagramType::ClassDiagram // default
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for line in code.lines() {
            let line = line.trim();
            if line.starts_with("class ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    nodes.push(Node {
                        id: parts[1].to_string(),
                        label: None,
                    });
                }
            } else if line.contains("-->") {
                let parts: Vec<&str> = line.split("-->").collect();
                if parts.len() == 2 {
                    let from = parts[0].trim().to_string();
                    let to = parts[1].trim().to_string();
                    edges.push(Edge {
                        from,
                        to,
                        label: None,
                    });
                }
            } else if line.contains("->") {
                let parts: Vec<&str> = line.split("->").collect();
                if parts.len() == 2 {
                    let from = parts[0].trim().to_string();
                    let to = parts[1].trim().to_string();
                    edges.push(Edge {
                        from,
                        to,
                        label: None,
                    });
                }
            }
        }

        Ok(MermaidAST {
            diagram_type,
            nodes,
            edges,
        })
    }

    pub fn validate(&self, ast: &MermaidAST, config: &Config) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Check for duplicate nodes
        if config.rules.get("duplicate-node").map(|r| r.enabled).unwrap_or(true) {
            let severity = config.rules.get("duplicate-node").map(|r| r.severity.clone()).unwrap_or(crate::config::Severity::Error);
            let mut seen_ids = std::collections::HashSet::new();
            for node in &ast.nodes {
                if !seen_ids.insert(&node.id) {
                    issues.push(LintIssue {
                        file: std::path::PathBuf::new(),
                        line: 0,
                        column: 0,
                        message: format!("Duplicate node ID: {}", node.id),
                        severity: severity.clone(),
                        rule: "duplicate-node".to_string(),
                        fix: None,
                    });
                }
            }
        }

        // Check for unknown nodes in edges
        if config.rules.get("unknown-node").map(|r| r.enabled).unwrap_or(true) {
            let severity = config.rules.get("unknown-node").map(|r| r.severity.clone()).unwrap_or(crate::config::Severity::Error);
            let node_ids: std::collections::HashSet<_> = ast.nodes.iter().map(|n| &n.id).collect();
            for edge in &ast.edges {
                if !node_ids.contains(&edge.from) {
                    issues.push(LintIssue {
                        file: std::path::PathBuf::new(),
                        line: 0,
                        column: 0,
                        message: format!("Edge references unknown node: {}", edge.from),
                        severity: severity.clone(),
                        rule: "unknown-node".to_string(),
                        fix: None,
                    });
                }
                if !node_ids.contains(&edge.to) {
                    issues.push(LintIssue {
                        file: std::path::PathBuf::new(),
                        line: 0,
                        column: 0,
                        message: format!("Edge references unknown node: {}", edge.to),
                        severity: severity.clone(),
                        rule: "unknown-node".to_string(),
                        fix: None,
                    });
                }
            }
        }

        // Check naming convention
        if config.rules.get("naming-convention").map(|r| r.enabled).unwrap_or(true) {
            let severity = config.rules.get("naming-convention").map(|r| r.severity.clone()).unwrap_or(crate::config::Severity::Warning);
            for node in &ast.nodes {
                if !is_pascal_case(&node.id) {
                    issues.push(LintIssue {
                        file: std::path::PathBuf::new(),
                        line: 0,
                        column: 0,
                        message: format!("Node '{}' should be PascalCase", node.id),
                        severity: severity.clone(),
                        rule: "naming-convention".to_string(),
                        fix: Some(pascal_case_fix(&node.id)),
                    });
                }
            }
        }

        // Check large diagram
        if config.rules.get("large-diagram").map(|r| r.enabled).unwrap_or(true) {
            let severity = config.rules.get("large-diagram").map(|r| r.severity.clone()).unwrap_or(crate::config::Severity::Info);
            if ast.nodes.len() > 10 {
                issues.push(LintIssue {
                    file: std::path::PathBuf::new(),
                    line: 0,
                    column: 0,
                    message: format!("Diagram has {} nodes, consider breaking into sub-diagrams", ast.nodes.len()),
                    severity: severity.clone(),
                    rule: "large-diagram".to_string(),
                    fix: None,
                });
            }
        }

        // Check missing edge labels
        if config.rules.get("missing-edge-label").map(|r| r.enabled).unwrap_or(false) {
            let severity = config.rules.get("missing-edge-label").map(|r| r.severity.clone()).unwrap_or(crate::config::Severity::Warning);
            for edge in &ast.edges {
                if edge.label.is_none() {
                    issues.push(LintIssue {
                        file: std::path::PathBuf::new(),
                        line: 0,
                        column: 0,
                        message: format!("Edge from '{}' to '{}' is missing a label", edge.from, edge.to),
                        severity: severity.clone(),
                        rule: "missing-edge-label".to_string(),
                        fix: None,
                    });
                }
            }
        }

        issues
    }
}

fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    chars.next().map_or(false, |c| c.is_uppercase()) && chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn pascal_case_fix(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize {
                result.extend(c.to_uppercase());
                capitalize = false;
            } else {
                result.extend(c.to_lowercase());
            }
        } else if c == '_' {
            capitalize = true;
        }
    }
    result
}

