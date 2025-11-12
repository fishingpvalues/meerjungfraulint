

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_mermaid_blocks(&self, content: &str) -> Vec<(usize, String)> {
        let mut blocks = Vec::new();
        let parser = Parser::new(content);
        let mut in_code_block = false;
        let mut code = String::new();
        let mut start_line = 0;

        for (event, range) in parser.into_offset_iter() {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                        if lang.as_ref() == "mermaid" {
                            in_code_block = true;
                            code.clear();
                            // Approximate line number
                            start_line = content[..range.start].chars().filter(|&c| c == '\n').count() + 1;
                        }
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        in_code_block = false;
                        blocks.push((start_line, code.clone()));
                    }
                }
                Event::Text(text) => {
                    if in_code_block {
                        code.push_str(&text);
                    }
                }
                _ => {}
            }
        }

        blocks
    }
}