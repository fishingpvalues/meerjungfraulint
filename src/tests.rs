#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::MarkdownParser;
    use crate::mermaid_parser::MermaidParser;
    use crate::config::Config;

    #[test]
    fn test_extract_mermaid_blocks() {
        let parser = MarkdownParser::new();
        let content = r#"
# Doc

Some text

```mermaid
classDiagram
class A
```

More text
"#;
        let blocks = parser.extract_mermaid_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].1.contains("classDiagram"));
    }

    #[test]
    fn test_parse_mermaid() {
        let parser = MermaidParser::new();
        let code = r#"
classDiagram
class A
class B
A --> B
"#;
        let ast = parser.parse(code).unwrap();
        assert_eq!(ast.nodes.len(), 2);
        assert_eq!(ast.edges.len(), 1);
    }

    #[test]
    fn test_validate_naming() {
        let parser = MermaidParser::new();
        let config = Config::default();
        let ast = crate::mermaid_parser::MermaidAST {
            diagram_type: crate::mermaid_parser::DiagramType::ClassDiagram,
            nodes: vec![crate::mermaid_parser::Node { id: "myClass".to_string(), label: None }],
            edges: vec![],
        };
        let issues = parser.validate(&ast, &config);
        assert!(issues.iter().any(|i| i.rule == "naming-convention"));
    }
}