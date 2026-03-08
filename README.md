# arxiv_mcp

MCP (Model Context Protocol) server for arXiv paper search and download.

## Features

- Search arXiv papers by keywords, title, abstract, or author
- Download paper PDFs
- MCP protocol implementation using [rmcp](https://github.com/mcp-sh/rmcp)

## Installation

### From Source

```bash
cargo build --release
```

### Using MCP Clients

This server can be used with MCP-compatible clients like:
- Claude Desktop
- Cursor
- Other MCP-enabled applications

## Usage

### Standalone Testing

```bash
# Test search functionality
cargo run --release -- --test
```

### MCP Server Mode

Run as an MCP server (stdin/stdout):

```bash
cargo run --release
```

### Configuration

The server uses default configuration. You can customize by modifying `src/models.rs`.

## Available Tools

- `search_papers`: Search arXiv papers with various filters
- `download_paper`: Download paper PDF by arXiv ID
- `search_and_download_papers`: Search and download multiple papers

## Requirements

- Rust 1.70+
- tokio
- reqwest
- scraper

## License

MIT
