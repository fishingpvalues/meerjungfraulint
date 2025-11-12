use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::cell::RefCell;
use crate::linter::Linter;
use crate::config::Config;

pub struct LSPServer {
    client: Client,
    documents: RefCell<HashMap<Url, String>>,
    linter: Linter,
}

impl LSPServer {
    pub async fn run() -> anyhow::Result<()> {
        let (service, socket) = LspService::new(|client| Self {
            client,
            documents: RefCell::new(HashMap::new()),
            linter: Linter::new(Config::load(None).unwrap_or_default()),
        });
        Server::new(tokio::io::stdin(), tokio::io::stdout(), socket).serve(service).await;
        Ok(())
    }

    async fn lint_document(&self, uri: &Url) -> Vec<Diagnostic> {
        if let Some(content) = self.documents.borrow().get(uri) {
            match self.linter.lint_content(content) {
                Ok(issues) => issues.into_iter().map(|issue| Diagnostic {
                    range: Range {
                        start: Position { line: issue.line as u32, character: issue.column as u32 },
                        end: Position { line: issue.line as u32, character: (issue.column + 1) as u32 },
                    },
                    severity: Some(match issue.severity {
                        crate::config::Severity::Error => DiagnosticSeverity::ERROR,
                        crate::config::Severity::Warning => DiagnosticSeverity::WARNING,
                        crate::config::Severity::Info => DiagnosticSeverity::INFORMATION,
                    }),
                    code: Some(NumberOrString::String(issue.rule)),
                    message: issue.message,
                    ..Default::default()
                }).collect(),
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LSPServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Lintjungfrau LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.borrow_mut().insert(params.text_document.uri.clone(), params.text_document.text);
        let diagnostics = self.lint_document(&params.text_document.uri).await;
        self.client.publish_diagnostics(params.text_document.uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.documents.borrow_mut().insert(params.text_document.uri.clone(), change.text.clone());
        }
        let diagnostics = self.lint_document(&params.text_document.uri).await;
        self.client.publish_diagnostics(params.text_document.uri, diagnostics, None).await;
    }
}

impl LSPServer {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: initialize properly
        Ok(Self {
            client: todo!(),
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // TODO: implement LSP server
        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LSPServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Lintjungfrau LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}