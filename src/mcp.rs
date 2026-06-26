use crate::ConfigBuilder;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct McpServer {
    host: String,
    port: u16,
}

impl McpServer {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port))
            .await
            .context("Failed to bind to address")?;

        println!("MCP server listening on {}:{}", self.host, self.port);

        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("Connection from {}", addr);

            tokio::spawn(async move {
                let mut buffer = vec![0u8; 1024];
                match socket.read(&mut buffer).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }
                        let request_str = String::from_utf8_lossy(&buffer[..n]);
                        if let Ok(request) = serde_json::from_str::<McpRequest>(&request_str) {
                            let response = handle_request(request).await;
                            let response_json = serde_json::to_string(&response).unwrap();
                            let _ = socket.write_all(response_json.as_bytes()).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                    }
                }
            });
        }
    }

    pub fn get_tools() -> Vec<ToolInfo> {
        vec![
            ToolInfo {
                name: "lint_files".to_string(),
                description: "Lint specified files and return issues".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "List of file or directory paths to lint"
                        },
                        "max_line_length": {
                            "type": "integer",
                            "description": "Maximum allowed line length"
                        },
                        "rules": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "List of rules to enable"
                        }
                    },
                    "required": ["paths"]
                }),
            },
            ToolInfo {
                name: "list_rules".to_string(),
                description: "List all available linting rules".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }
}

async fn handle_request(request: McpRequest) -> McpResponse {
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(request.params).await,
        _ => Err(McpError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    match result {
        Ok(value) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    }
}

fn handle_initialize() -> Result<Value, McpError> {
    Ok(serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "lint-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

fn handle_tools_list() -> Result<Value, McpError> {
    let tools = McpServer::get_tools();
    Ok(serde_json::json!({
        "tools": tools
    }))
}

async fn handle_tools_call(params: Option<Value>) -> Result<Value, McpError> {
    let params = params.ok_or(McpError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(McpError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

    let arguments = params.get("arguments");

    match tool_name {
        "lint_files" => execute_lint(arguments).await,
        "list_rules" => execute_list_rules(),
        _ => Err(McpError {
            code: -32601,
            message: format!("Tool not found: {}", tool_name),
            data: None,
        }),
    }
}

async fn execute_lint(arguments: Option<&Value>) -> Result<Value, McpError> {
    let args = arguments.ok_or(McpError {
        code: -32602,
        message: "Missing arguments".to_string(),
        data: None,
    })?;

    let paths = args
        .get("paths")
        .and_then(|v| v.as_array())
        .ok_or(McpError {
            code: -32602,
            message: "Missing paths array".to_string(),
            data: None,
        })?;

    let mut builder = ConfigBuilder::new();
    let mut paths_vec = Vec::new();

    for path in paths {
        if let Some(path_str) = path.as_str() {
            paths_vec.push(std::path::PathBuf::from(path_str));
        }
    }

    builder = builder.paths(paths_vec);

    if let Some(length) = args.get("max_line_length").and_then(|v| v.as_u64()) {
        builder = builder.max_line_length(Some(length as usize));
    }

    if let Some(rules) = args.get("rules").and_then(|v| v.as_array()) {
        let rules_vec: Vec<String> = rules
            .iter()
            .filter_map(|r| r.as_str().map(|s| s.to_string()))
            .collect();
        builder = builder.enabled_rules(rules_vec);
    }

    let config = builder.build();

    match crate::lint_files(&config) {
        Ok(results) => {
            let issues: Vec<Value> = results
                .into_iter()
                .flat_map(|r| {
                    let file_path = r.file_path;
                    r.messages.into_iter().map(move |m| {
                        serde_json::json!({
                            "file": file_path,
                            "line": m.line,
                            "column": m.column,
                            "severity": m.severity.as_str(),
                            "message": m.message,
                            "rule": m.rule,
                            "suggestion": m.suggestion
                        })
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("Found {} issues", issues.len())
                    }
                ],
                "issues": issues
            }))
        }
        Err(e) => Err(McpError {
            code: -32603,
            message: format!("Linting error: {}", e),
            data: None,
        }),
    }
}

fn execute_list_rules() -> Result<Value, McpError> {
    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": "Available rules:\n- line-length: Check for lines exceeding maximum length\n- trailing-whitespace: Check for trailing whitespace\n- no-todo: Check for TODO/FIXME comments"
            }
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new("127.0.0.1".to_string(), 8080);
        assert_eq!(server.host, "127.0.0.1");
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_mcp_server_get_tools() {
        let tools = McpServer::get_tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "lint_files");
        assert_eq!(tools[1].name, "list_rules");
    }

    #[test]
    fn test_handle_initialize() {
        let result = handle_initialize();
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("protocolVersion").is_some());
        assert!(value.get("capabilities").is_some());
    }

    #[test]
    fn test_handle_tools_list() {
        let result = handle_tools_list();
        assert!(result.is_ok());

        let value = result.unwrap();
        let tools = value.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_mcp_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "initialize".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("initialize"));
        assert!(json.contains("2.0"));
    }

    #[test]
    fn test_mcp_response_serialization() {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("result"));
    }

    #[test]
    fn test_mcp_error_serialization() {
        let error = McpError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("-32601"));
        assert!(json.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_execute_list_rules() {
        let result = execute_list_rules();
        assert!(result.is_ok());

        let value = result.unwrap();
        let content = value.get("content").unwrap().as_array().unwrap();
        assert_eq!(content.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_lint_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.rs");
        std::fs::write(&path, "fn main() { let x = 5; }").unwrap();

        let args = serde_json::json!({
            "paths": [path.to_string_lossy()],
            "rules": ["line-length"]
        });

        let result = execute_lint(Some(&args)).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.get("issues").is_some());
        assert!(value.get("content").is_some());
    }

    #[tokio::test]
    async fn test_execute_lint_missing_paths() {
        let args = serde_json::json!({});
        let result = execute_lint(Some(&args)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_lint_nonexistent_path() {
        let args = serde_json::json!({
            "paths": ["/nonexistent/path/file.rs"]
        });
        let result = execute_lint(Some(&args)).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let issues = value.get("issues").unwrap().as_array().unwrap();
        assert!(issues.is_empty());
    }
}
