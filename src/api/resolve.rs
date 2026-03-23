use crate::api::gamma::fetch_market_by_slug;

pub async fn resolve_result(
    market_id: &str,
    up_price: f64,
    down_price: f64,
) -> String {
    let url = format!(
        "https://gamma-api.polymarket.com/markets/{}",
        market_id
    );

    if let Ok(res) = reqwest::get(&url).await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if json["closed"].as_bool() == Some(true) {
                if let Some(prices) = json["outcomePrices"].as_array() {
                    let p0 = prices[0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    let p1 = prices[1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                    return if p0 > p1 { "UP".to_string() } else { "DOWN".to_string() };
                }
            }
        }
    }

    // fallback
    if up_price > down_price {
        "UP".to_string()
    } else {
        "DOWN".to_string()
    }
}