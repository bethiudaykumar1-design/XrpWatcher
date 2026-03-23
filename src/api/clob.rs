// use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// struct MidpointResponse {
//     mid: Option<String>,
// }

// #[derive(Debug, Deserialize)]
// struct LastTradeResponse {
//     price: Option<String>,
// }

// /// Fetch price using midpoint API
// async fn fetch_midpoint(token_id: &str) -> Option<f64> {
//     let url = format!(
//         "https://clob.polymarket.com/midpoint?token_id={}",
//         token_id
//     );

//     let client = reqwest::Client::new();

//     let res = client.get(&url).send().await.ok()?;

//     let data: MidpointResponse = res.json().await.ok()?;

//     data.mid?.parse::<f64>().ok()
// }

// /// Fallback: last trade price
// async fn fetch_last_trade(token_id: &str) -> Option<f64> {
//     let url = format!(
//         "https://clob.polymarket.com/last-trade-price?token_id={}",
//         token_id
//     );

//     let client = reqwest::Client::new();

//     let res = client.get(&url).send().await.ok()?;

//     let data: LastTradeResponse = res.json().await.ok()?;

//     data.price?.parse::<f64>().ok()
// }

// /// Public function: get price with fallback
// pub async fn get_price(token_id: &str) -> Option<f64> {
//     if let Some(price) = fetch_midpoint(token_id).await {
//         return Some(price);
//     }

//     println!("⚠️ Midpoint failed, using fallback...");

//     fetch_last_trade(token_id).await
// }

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MidpointResponse {
    mid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LastTradeResponse {
    price: Option<String>,
}

/// Fetch price using midpoint API (returns None if not available)
pub async fn fetch_midpoint(token_id: &str) -> Option<f64> {
    let url = format!(
        "https://clob.polymarket.com/midpoint?token_id={}",
        token_id
    );

    let client = reqwest::Client::new();

    match client.get(&url).send().await {
        Ok(res) => {
            if let Ok(data) = res.json::<MidpointResponse>().await {
                if let Some(mid) = data.mid {
                    if let Ok(price) = mid.parse::<f64>() {
                        return Some(price);
                    }
                }
            }
            None
        }
        Err(e) => {
            println!("⚠️ Failed to fetch midpoint for {}: {}", token_id, e);
            None
        }
    }
}

/// Fetch last trade price using last trade API
pub async fn fetch_last_trade(token_id: &str) -> Option<f64> {
    let url = format!(
        "https://clob.polymarket.com/last-trade?token_id={}",
        token_id
    );

    let client = reqwest::Client::new();

    match client.get(&url).send().await {
        Ok(res) => {
            if let Ok(data) = res.json::<LastTradeResponse>().await {
                if let Some(price_str) = data.price {
                    if let Ok(price) = price_str.parse::<f64>() {
                        return Some(price);
                    }
                }
            }
            None
        }
        Err(e) => {
            println!("⚠️ Failed to fetch last trade for {}: {}", token_id, e);
            None
        }
    }
}

/// Fetch initial price with fallback: try midpoint first, then last trade
pub async fn fetch_initial_price(token_id: &str) -> f64 {
    // Try midpoint first
    if let Some(price) = fetch_midpoint(token_id).await {
        println!("✅ Got midpoint price: {:.4} for {}", price, token_id);
        return price;
    }
    
    // Fallback to last trade
    if let Some(price) = fetch_last_trade(token_id).await {
        println!("✅ Got last trade price: {:.4} for {}", price, token_id);
        return price;
    }
    
    // Default if both fail
    println!("⚠️ Could not fetch price for {}, using 0.5", token_id);
    0.5 // Reasonable default for binary markets
}