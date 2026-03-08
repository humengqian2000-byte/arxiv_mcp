use crate::error::ArxivError;
use crate::models::{Config, Paper};
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::{
        Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::arxiv::ArxivClient;

fn default_search_type() -> String {
    "all".to_string()
}

fn default_output_dir() -> String {
    "./papers".to_string()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchArgs {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub searchtype: String,
    pub limit: Option<usize>,
    #[serde(default)]
    pub start: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DownloadArgs {
    pub paper_id: String,
    pub output_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchAndDownloadArgs {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub searchtype: String,
    pub limit: Option<usize>,
    #[serde(default)]
    pub start: Option<usize>,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

#[derive(Clone)]
pub struct ArxivServer {
    client: Arc<Mutex<ArxivClient>>,
    tool_router: ToolRouter<ArxivServer>,
}

#[tool_router]
impl ArxivServer {
    pub fn new(config: Config) -> Self {
        let client = ArxivClient::new(config);
        Self {
            client: Arc::new(Mutex::new(client)),
            tool_router: Self::tool_router(),
        }
    }

    pub async fn test_search(
        &self,
        query: &str,
        searchtype: &str,
        limit: Option<usize>,
        start: Option<usize>,
    ) -> Result<Vec<Paper>, ArxivError> {
        let client = self.client.lock().await;
        client.search(query, searchtype, limit, start).await
    }

    #[tool(description = "Search for academic papers on arXiv\n\nTip: If user query contains typos or is not in English, please improve/translate it for better search results.\n\nParameters:\n- query: Search keywords (AI will optimize if needed)\n- searchtype: Search type ('all', 'title', 'abstract', 'author')\n- start: Starting position for pagination (0, 10, 20, ...)\n- limit: Number of results to return (default: 10)\n\nNote: start=0 returns first 10 results, start=10 returns results 11-20, etc.")]
    async fn search_papers(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        let client = self.client.lock().await;
        let papers = client
            .search(&args.query, &args.searchtype, args.limit, args.start)
            .await
            .map_err(|e| McpError::internal_error(
                std::borrow::Cow::Owned(e.to_string()),
                None
            ))?;

        Ok(rmcp::model::CallToolResult::success(vec![
            rmcp::model::Content::text(serde_json::to_string_pretty(&papers).unwrap())
        ]))
    }

    #[tool(description = "Download arXiv paper as PDF\n\nParameters:\n- paper_id: ArXiv paper ID (e.g., '2401.12345')\n- output_dir: Output directory (default: './papers' in current project root)\n\nReturns: JSON with file_path where the PDF was saved.")]
    async fn download_paper(
        &self,
        Parameters(args): Parameters<DownloadArgs>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        let client = self.client.lock().await;
        let bytes = client
            .download_pdf(&args.paper_id)
            .await
            .map_err(|e| McpError::internal_error(
                std::borrow::Cow::Owned(e.to_string()),
                None
            ))?;

        // Determine output directory (default: ./papers)
        let output_dir = args.output_dir.unwrap_or_else(|| "./papers".to_string());
        let file_path = format!("{}/{}.pdf", output_dir, args.paper_id);

        // Create directory if not exists
        std::fs::create_dir_all(&output_dir).map_err(|e| McpError::internal_error(
            std::borrow::Cow::Owned(format!("Failed to create directory: {}", e)),
            None
        ))?;

        // Write file
        std::fs::write(&file_path, &bytes).map_err(|e| McpError::internal_error(
            std::borrow::Cow::Owned(format!("Failed to write file: {}", e)),
            None
        ))?;

        Ok(rmcp::model::CallToolResult::success(vec![
            rmcp::model::Content::text(serde_json::json!({
                "paper_id": args.paper_id,
                "file_path": file_path,
                "size_bytes": bytes.len()
            }).to_string())
        ]))
    }

    #[tool(description = "Search papers and download PDFs from arXiv\n\nTip: If user query contains typos or is not in English, please improve/translate it for better search results.\n\nParameters:\n- query: Search keywords (AI will optimize if needed)\n- searchtype: Search type ('all', 'title', 'abstract', 'author')\n- start: Starting position for pagination (0, 10, 20, ...)\n- limit: Number of results to return (default: 10)\n- output_dir: Output directory (default: './papers')\n\nReturns: JSON with papers including content and file paths, plus manifest saved to output_dir.")]
    async fn search_and_download_papers(
        &self,
        Parameters(args): Parameters<SearchAndDownloadArgs>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        let client = self.client.lock().await;

        // 1. Search papers
        let papers = client
            .search(&args.query, &args.searchtype, args.limit, args.start)
            .await
            .map_err(|e| McpError::internal_error(
                std::borrow::Cow::Owned(e.to_string()),
                None
            ))?;

        // Determine output directory
        let output_dir = &args.output_dir;

        // Create directory if not exists
        std::fs::create_dir_all(output_dir).map_err(|e| McpError::internal_error(
            std::borrow::Cow::Owned(format!("Failed to create directory: {}", e)),
            None
        ))?;

        // 2. Download each paper
        let mut downloaded_papers = Vec::new();
        for mut paper in papers {
            match client.download_pdf(&paper.id).await {
                Ok(bytes) => {
                    let file_path = format!("{}/{}.pdf", output_dir, paper.id);
                    if let Err(e) = std::fs::write(&file_path, &bytes) {
                        tracing::warn!("Failed to write file {}: {}", file_path, e);
                    } else {
                        paper.file_path = Some(file_path);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to download {}: {}", paper.id, e);
                }
            }
            downloaded_papers.push(paper);
        }

        // 3. Generate/append manifest
        let manifest_path = format!("{}/README.md", output_dir);
        update_manifest(&manifest_path, &downloaded_papers).map_err(|e| McpError::internal_error(
            std::borrow::Cow::Owned(format!("Failed to update manifest: {}", e)),
            None
        ))?;

        Ok(rmcp::model::CallToolResult::success(vec![
            rmcp::model::Content::text(serde_json::json!({
                "papers": downloaded_papers,
                "manifest_path": manifest_path
            }).to_string())
        ]))
    }
}

/// Update manifest file with new papers (append if exists)
fn update_manifest(manifest_path: &str, papers: &[Paper]) -> std::io::Result<()> {
    let mut content = String::new();

    // Read existing manifest if exists
    if std::path::Path::new(manifest_path).exists() {
        content = std::fs::read_to_string(manifest_path)?;
    } else {
        // Create new manifest header
        content.push_str("# ArXiv Papers\n\n");
    }

    // Append new papers that don't exist in manifest
    for paper in papers {
        let paper_title = &paper.title;
        if !content.contains(paper_title) {
            content.push_str(&format!("## {}\n\n", paper_title));
            content.push_str(&format!("- **ID**: {}\n", paper.id));
            content.push_str(&format!("- **Authors**: {}\n", paper.authors.join(", ")));
            content.push_str(&format!("- **Categories**: {}\n", paper.categories.join(", ")));
            if let Some(ref file_path) = paper.file_path {
                let file_name = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                content.push_str(&format!("- **File**: [{}]({})\n", file_name, file_path));
            }
            content.push_str(&format!("- **Abstract**: {}\n\n", paper.abstract_text));
        }
    }

    std::fs::write(manifest_path, content)
}

#[tool_handler]
impl ServerHandler for ArxivServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_protocol_version(ProtocolVersion::V_2024_11_05)
        .with_instructions("ArXiv MCP Server - Search papers and download PDFs".to_string())
    }
}

