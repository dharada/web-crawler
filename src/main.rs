use log::{debug, error, info, warn};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashSet;
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::sync::{Arc, Mutex};
use url::Url;

const MAX_SEGMENTS: usize = 3;
const DEFAULT_MAX_DEPTH: usize = 5;

fn default_max_depth() -> usize {
    DEFAULT_MAX_DEPTH
}

#[derive(Deserialize)]
struct AppConfig {
    start_urls: Vec<String>,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger: Write to Terminal AND "crawler.log" file
    CombinedLogger::init(vec![
        // Log to terminal
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // Log to file
        WriteLogger::new(
            LevelFilter::Info,
            //LevelFilter::Debug,
            Config::default(),
            File::create("my-web-crawler.log").unwrap(),
        ),
    ])
    .unwrap();

    if fs::metadata("crawled_pages").is_ok() {
        fs::remove_dir_all("crawled_pages")?;
    }
    fs::create_dir_all("crawled_pages")?;

    // Load configuration from config file
    let file = File::open("config.json")?;
    let reader = BufReader::new(file);
    let config: AppConfig = serde_json::from_reader(reader)?;

    let mut start_urls = config.start_urls;
    let max_depth = config.max_depth;
    // Deduplicate and sort start URLs to avoid processing the same URL multiple times
    // start_urls.sort();
    start_urls.dedup();

    let visited = Arc::new(Mutex::new(HashSet::new()));
    let client = Client::new();

    for url in start_urls {
        crawl(client.clone(), Url::parse(&url)?, visited.clone(), 0, max_depth).await;
    }

    Ok(())
}

async fn crawl(
    client: Client,
    url: Url,
    visited: Arc<Mutex<HashSet<String>>>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    let mut visited_lock = visited.lock().unwrap();
    if visited_lock.contains(url.as_str()) {
        return;
    }
    visited_lock.insert(url.to_string());
    drop(visited_lock);

    //println!("Crawling: {}", url);
    info!("Crawling: {}", url);

    match client.get(url.as_str()).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(body) = response.text().await {
                    save_to_file(&url, &body);
                    parse_links(&client, &url, &body, visited.clone(), depth, max_depth).await;
                }
            } else {
                error!("Failed to fetch {}: Status {}", url, response.status());
            }
        }
        Err(e) => {
            error!("Failed to fetch {}: {}", url, e);
        }
    }
}

fn save_to_file(url: &Url, html: &str) {
    let document = Html::parse_document(html);
    let selector = Selector::parse("main").unwrap();

    if let Some(body) = document.select(&selector).next() {
        // URL全体を文字列化し、プロトコル削除後、（英数字とピリオド）以外を "_" に置換
        let raw_filename = url
            .to_string()
            .replace("https://", "")
            .replace("http://", "")
            .replace(|c: char| !c.is_alphanumeric() && !c.eq(&'.'), "_");

        // "_" で分割してセグメントを取得
        let segments: Vec<&str> = raw_filename.split('_').filter(|s| !s.is_empty()).collect();

        // Rule1 & Rule2 に基づくファイル名の決定
        let filename = if segments.len() <= MAX_SEGMENTS {
            // Rule1: セグメント数がMAX_SEGMENTS以内の場合はファイルをマージしない (全セグメントを使用)
            segments.join("_")
        } else {
            // Rule2: セグメント数がMAX_SEGMENTSを超える場合は、MAX_SEGMENTS番目セグメントまでのパスでマージする
            segments[..MAX_SEGMENTS].join("_")
        };

        let file_path = format!("crawled_pages/{}.txt", filename);

        // 追記モードでファイルを開く
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) {
            // 区切り線とURLをヘッダーとして書き込む
            let header = format!("\n\n========================================\nURL: {}\n========================================\n", url);
            let _ = file.write_all(header.as_bytes());
            let _ = file.write_all(body.inner_html().as_bytes());
        }
    }
}

async fn parse_links(
    client: &Client,
    base_url: &Url,
    html: &str,
    visited: Arc<Mutex<HashSet<String>>>,
    depth: usize,
    max_depth: usize,
) {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").unwrap();

    let mut links_to_visit = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            debug!("Found link: {}, depth: {}", href, depth);

            if let Ok(mut absolute_url) = base_url.join(href) {
                debug!("Absolute URL: {}, depth: {}", absolute_url, depth);
                // Remove fragment (anchor) to ensure we crawl the page, not just a section
                absolute_url.set_fragment(None);

                // Check if the link is within the same domain
                if absolute_url.domain() == base_url.domain() {
                    links_to_visit.push(absolute_url);
                } else {
                    debug!(
                        "Skip as the Link is external to this domain. {}",
                        absolute_url
                    );
                }
            } else {
                warn!("Failed to join URL with base URL. Found link is {}", href);
            }
        }
    }

    // Deduplicate links to avoid processing the same URL multiple times from this page
    links_to_visit.sort();
    links_to_visit.dedup();

    for link in links_to_visit {
        let visited_clone = visited.clone();
        let client_clone = client.clone();
        // Recursive call with depth increment
        // Box::pin is required to handle recursion in async functions
        Box::pin(crawl(client_clone, link, visited_clone, depth + 1, max_depth)).await;
    }
}
