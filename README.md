# My Web Crawler

A simple, asynchronous web crawler written in Rust. This application visits specified URLs, scrapes the content, and saves the main text body into local text files. It supports recursive crawling up to a defined depth.

## Features

- **Async Crawling**: Built with `tokio` and `reqwest` for efficient, non-blocking HTTP requests.
- **Configurable Entry Points**: Load target URLs from an external JSON configuration file (`config.json`).
- **Content Extraction**: Uses `scraper` to extract the main content of pages.
- **Logging**: Dual logging to both the console (stdout) and a file (`my-web-crawler.log`).
- **Deduplication**: Avoids visiting the same URL twice to prevent infinite loops.

## Prerequisites

You need to have **Rust** and **Cargo** installed on your machine.
If you haven't installed them yet, you can do so via [rustup.rs](https://rustup.rs/).

## Configuration

Before running this crawler, ensure you have a `config.json` file in the root directory of the project. This file defines the starting URLs for the crawler.

**`config.json` example:**
```
json
{
  "start_urls": [
    "https://docs.mulesoft.com/release-notes/index",
    "https://docs.mulesoft.com/general/glossary"
  ]
}
```

## How to Build and Execute

### 1. Build the project

To compile the project and download dependencies:
```
bash
cargo build
```
For a production-ready build (optimized for speed, recommended for large crawls):
```
bash
cargo build --release
```
### 2. Run the crawler

To run the crawler directly:
```
bash
cargo run
```
If you want to run the optimized release version:
```
bash
cargo run --release
```
## Output

*   **Crawled Data**: The scraped content is saved in the `crawled_pages/` directory.
    *   Files are named based on URL segments.
    *   If a file for a specific URL structure already exists, the new content is **appended** to it (preserving previous data).
*   **Logs**: A detailed log of the execution (including debug information) is written to `my-web-crawler.log` in the project root. Basic info is also printed to the terminal.

## Project Structure

*   `src/main.rs`: Main application logic.
*   `config.json`: Configuration file for target URLs.
*   `crawled_pages/`: Directory where output text files are stored.
*   `my-web-crawler.log`: Execution log file created at runtime.
```