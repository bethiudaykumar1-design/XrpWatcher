use serde::Deserialize;

use tokio::time::{sleep,Duration};

#[derive(Debug, Deserialize)]
pub struct Market {
    pub id: String,
    pub question: String,
    #[serde(rename = "clobTokenIds")]
    pub clob_token_ids: String,
    pub outcomes: String,
    pub active: bool,
    pub closed: bool,
}

/// Fetch market by slug
pub async fn fetch_market_by_slug(slug: &str) -> Option<Market> {
    let url = format!(
        "https://gamma-api.polymarket.com/markets?slug={}",
        slug
    );

    let client = reqwest::Client::new();

    let res = client.get(&url).send().await.ok()?;

    let markets: Vec<Market> = res.json().await.ok()?;

    markets.into_iter().next()
}

/// Extract UP and DOWN token IDs
pub fn extract_token_ids(market: &Market) -> Option<(String, String)> {
    let tokens: Vec<String> = serde_json::from_str(&market.clob_token_ids).ok()?;
    let outcomes: Vec<String> = serde_json::from_str(&market.outcomes).ok()?;

    if tokens.len() != 2 || outcomes.len() != 2 {
        return None;
    }

    let mut up_token = String::new();
    let mut down_token = String::new();

    for (i, outcome) in outcomes.iter().enumerate() {
        if outcome.to_lowercase() == "up" {
            up_token = tokens[i].clone();
        } else if outcome.to_lowercase() == "down" {
            down_token = tokens[i].clone();
        }
    }

    if up_token.is_empty() || down_token.is_empty() {
        return None;
    }

    Some((up_token, down_token))
}


pub async fn fetch_with_retry(slug: &str) -> Option<Market> {
    for attempt in 1..=10 {
        // println!("Fetching market (attempt {})...", attempt);

        if let Some(market) = fetch_market_by_slug(slug).await {
            println!("✅ Market found!");
            return Some(market);
        }

        println!("⏳ Market not available yet, retrying...");
        sleep(Duration::from_secs(3)).await;
    }

    println!("❌ Failed to fetch market after retries");
    None
}