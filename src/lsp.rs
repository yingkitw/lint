use crate::{ConfigBuilder, Linter};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<LspError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspError {
    code: i32,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct InitializeParams {
    #[serde(rename = "processId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    process_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InitializeResult {
    capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerCapabilities {
    #[serde(rename = "textDocumentSync")]
    text_document_sync: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DidChangeTextDocumentParams {
    #[serde(rename = "textDocument")]
    text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentItem {
    uri: String,
    #[serde(rename = "languageId")]
    language_id: String,
    version: i32,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionedTextDocumentIdentifier {
    uri: String,
    version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentContentChangeEvent {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublishDiagnosticsParams {
    uri: String,
    diagnostics: Vec<Diagnostic>,
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Diagnostic {
    range: Range,
    severity: i32,
    message: String,
    source: String,
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    line: u32,
    character: u32,
}

fn severity_to_lsp(sev: &crate::output::Severity) -> i32 {
    match sev {
        crate::output::Severity::Error => 1,
        crate::output::Severity::Warning => 2,
        crate::output::Severity::Info => 3,
    }
}

pub struct LspServer;

impl LspServer {
    pub async fn run() -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout;

        loop {
            let msg = match Self::read_message(&mut reader).await {
                Ok(Some(m)) => m,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("LSP read error: {}", e);
                    continue;
                }
            };

            if let Err(e) = Self::handle_message(&mut stdout, msg).await {
                eprintln!("LSP handle error: {}", e);
            }
        }

        Ok(())
    }

    async fn read_message<R: AsyncBufReadExt + Unpin>(
        reader: &mut R,
    ) -> Result<Option<LspRequest>> {
        let mut header = String::new();
        let mut content_length: Option<usize> = None;

        loop {
            header.clear();
            if reader.read_line(&mut header).await? == 0 {
                return Ok(None);
            }
            let line = header.trim();
            if line.is_empty() {
                break;
            }
            if let Some(prefix) = line.strip_prefix("Content-Length: ") {
                content_length = prefix.parse().ok();
            }
        }

        let len = match content_length {
            Some(l) => l,
            None => return Ok(None),
        };

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;

        let req: LspRequest = serde_json::from_slice(&buf)?;
        Ok(Some(req))
    }

    async fn send_message<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        msg: &impl Serialize,
    ) -> Result<()> {
        let body = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        writer.write_all(header.as_bytes()).await?;
        writer.write_all(body.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn handle_message<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        req: LspRequest,
    ) -> Result<()> {
        match req.method.as_str() {
            "initialize" => {
                let result = InitializeResult {
                    capabilities: ServerCapabilities {
                        text_document_sync: 1, // Full document sync
                    },
                    server_info: Some(ServerInfo {
                        name: "lint-lsp".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    }),
                };
                let resp = LspResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::to_value(result)?),
                    error: None,
                };
                Self::send_message(writer, &resp).await?;
            }
            "initialized" | "$/setTrace" => {
                // No response needed for notifications
            }
            "textDocument/didOpen" => {
                if let Some(params) = req.params
                    && let Ok(p) = serde_json::from_value::<DidOpenTextDocumentParams>(params)
                {
                    Self::publish_diagnostics(
                        writer,
                        &p.text_document.uri,
                        &p.text_document.text,
                        Some(p.text_document.version),
                    )
                    .await?;
                }
            }
            "textDocument/didChange" => {
                if let Some(params) = req.params
                    && let Ok(p) = serde_json::from_value::<DidChangeTextDocumentParams>(params)
                    && let Some(last) = p.content_changes.last()
                {
                    Self::publish_diagnostics(
                        writer,
                        &p.text_document.uri,
                        &last.text,
                        Some(p.text_document.version),
                    )
                    .await?;
                }
            }
            "textDocument/didClose" => {
                if let Some(params) = req.params
                    && let Ok(p) = serde_json::from_value::<DidCloseTextDocumentParams>(params)
                {
                    let empty = PublishDiagnosticsParams {
                        uri: p.text_document.uri,
                        diagnostics: vec![],
                        version: None,
                    };
                    let notif = LspNotification {
                        jsonrpc: "2.0".to_string(),
                        method: "textDocument/publishDiagnostics".to_string(),
                        params: Some(serde_json::to_value(empty)?),
                    };
                    Self::send_message(writer, &notif).await?;
                }
            }
            "shutdown" => {
                let resp = LspResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(Value::Null),
                    error: None,
                };
                Self::send_message(writer, &resp).await?;
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(())
    }

    async fn publish_diagnostics<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        uri: &str,
        text: &str,
        version: Option<i32>,
    ) -> Result<()> {
        let diagnostics = Self::lint_text(uri, text);
        let params = PublishDiagnosticsParams {
            uri: uri.to_string(),
            diagnostics,
            version,
        };
        let notif = LspNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        Self::send_message(writer, &notif).await?;
        Ok(())
    }

    fn lint_text(uri: &str, text: &str) -> Vec<Diagnostic> {
        let mut config = ConfigBuilder::new().build();
        let path = std::path::PathBuf::from(uri.strip_prefix("file://").unwrap_or(uri));
        config.paths = vec![path.clone()];
        config.stdin_file_path = Some(path.to_string_lossy().to_string());

        let linter = Linter::new(&config);
        let mut result = match linter.lint_content(text, &path) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        result
            .messages
            .drain(..)
            .map(|msg| Diagnostic {
                range: Range {
                    start: Position {
                        line: (msg.line.saturating_sub(1)) as u32,
                        character: (msg.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (msg.line.saturating_sub(1)) as u32,
                        character: (msg.column.saturating_sub(1)) as u32,
                    },
                },
                severity: severity_to_lsp(&msg.severity),
                message: msg.message,
                source: "lint".to_string(),
                code: msg.rule,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DidCloseTextDocumentParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentIdentifier {
    uri: String,
}
