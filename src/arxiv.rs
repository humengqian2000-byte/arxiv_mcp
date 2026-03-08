use crate::error::ArxivError;
use crate::models::{Config, Paper};
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;

pub struct ArxivClient {
    client: Client,
    config: Arc<Config>,
}

impl ArxivClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            config: Arc::new(config),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        searchtype: &str,
        limit: Option<usize>,
        start: Option<usize>,
    ) -> Result<Vec<Paper>, ArxivError> {
        let mut url = format!(
            "https://arxiv.org/search/?query={}&searchtype={}&source=header",
            urlencoding::encode(query),
            urlencoding::encode(searchtype)
        );

        if let Some(start) = start {
            url.push_str(&format!("&start={}", start));
        }

        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;

        self.parse_papers(&body, limit)
    }

    fn parse_papers(&self, html: &str, limit: Option<usize>) -> Result<Vec<Paper>, ArxivError> {
        let document = Html::parse_document(html);
        let result_selector = Selector::parse("li.arxiv-result").unwrap();

        let results: Vec<_> = document.select(&result_selector).collect();
        let mut papers = Vec::new();
        let limit = limit.unwrap_or(50);

        for element in results.iter().take(limit) {
            // Extract arxiv ID
            let id = element
                .select(&Selector::parse(".list-title a").unwrap())
                .next()
                .and_then(|el| el.value().attr("href"))
                .and_then(|href| href.split('/').last())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if id.is_empty() {
                continue;
            }

            // Extract title
            let title = element
                .select(&Selector::parse("p.title.is-5").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Extract authors
            let authors: Vec<String> = element
                .select(&Selector::parse("p.authors a").unwrap())
                .map(|el| el.text().collect::<String>())
                .collect();

            // Extract abstract
            let abstract_text = element
                .select(&Selector::parse("span.abstract-short").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Extract categories
            let categories: Vec<String> = element
                .select(&Selector::parse("span.tag").unwrap())
                .map(|el| el.text().collect::<String>())
                .collect();

            // PDF URL
            let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", id);

            papers.push(Paper {
                id,
                title,
                authors,
                abstract_text,
                categories,
                pdf_url,
                file_path: None,
            });
        }

        Ok(papers)
    }

    pub async fn download_pdf(&self, paper_id: &str) -> Result<Vec<u8>, ArxivError> {
        let url = format!("https://arxiv.org/pdf/{}.pdf", paper_id);
        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Err(ArxivError::PaperNotFound(paper_id.to_string()));
        }

        let bytes = response.bytes().await?.to_vec();
        Ok(bytes)
    }
}
