use arxiv_mcp::models::Config;
use arxiv_mcp::server::ArxivServer;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 测试模式：如果传入 --test 参数，直接测试搜索功能
    if std::env::args().any(|arg| arg == "--test") {
        let config = Config::default();
        let server = ArxivServer::new(config);

        // Test search
        let papers = server.test_search("machine learning", "all", Some(3), None).await?;

        println!("Found {} papers:", papers.len());
        for paper in &papers {
            println!("- {} ({})", paper.title, paper.id);
            println!("  Abstract: {}...", paper.abstract_text.chars().take(100).collect::<String>());
            println!();
        }

        return Ok(());
    }

    let config = Config::default();
    let server = ArxivServer::new(config);

    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
