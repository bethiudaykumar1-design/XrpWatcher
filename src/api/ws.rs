// use std::collections::HashMap;
// use std::sync::Arc;

// use futures_util::{SinkExt, StreamExt};
// use serde_json::json;
// use tokio::sync::Mutex;
// use tokio_tungstenite::connect_async;
// use url::Url;

// use crate::models::market::MarketData;

// pub async fn start_ws(
//     up_id: String,
//     down_id: String,
//     market_data: Arc<Mutex<MarketData>>,
//     expected_market_id: String,
// ) {
//     loop {
//         println!("🔌 Connecting WebSocket...");

//         let url = Url::parse("wss://ws-subscriptions-clob.polymarket.com/ws/market").unwrap();

//         match connect_async(url).await {
//             Ok((mut ws, _)) => {
//                 println!("✅ WS Connected");

//                 let sub = json!({
//                     "assets_ids": [up_id, down_id],
//                     "type": "market"
//                 });

//                 ws.send(
//                     tokio_tungstenite::tungstenite::Message::Text(sub.to_string()),
//                 )
//                 .await
//                 .unwrap();

//                 let mut prices: HashMap<String, f64> = HashMap::new();

//                 while let Some(msg) = ws.next().await {
//                     let msg = match msg {
//                         Ok(m) => m,
//                         Err(_) => break, // reconnect
//                     };

//                     if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
//                         let ws_data: serde_json::Value = match serde_json::from_str(&txt) {
//                             Ok(v) => v,
//                             Err(_) => continue,
//                         };

//                         if ws_data["event_type"] == "last_trade_price" {
//                             let asset = match ws_data["asset_id"].as_str() {
//                                 Some(a) => a,
//                                 None => continue,
//                             };

//                             let price = match ws_data["price"].as_str() {
//                                 Some(p) => p.parse::<f64>().ok(),
//                                 None => None,
//                             };

//                             if let Some(price) = price {
//                                 prices.insert(asset.to_string(), price);

//                                 // ✅ Update MarketData when both available
//                                 if let (Some(up), Some(down)) =
//                                     (prices.get(&up_id), prices.get(&down_id))
//                                 {
//                                     // First, check if market has expired (without holding the lock)
//                                     let should_stop = {
//                                         let data = market_data.lock().await;
//                                         let now = crate::utils::time::now_ts();
//                                         if now > data.end_ts + 5 {
//                                             // Give 5 second grace period
//                                             println!("🛑 WS stopped (market expired)");
//                                             true
//                                         } else if data.market_id != expected_market_id {
//                                             println!("🛑 WS stopped (market changed)");
//                                             true
//                                         } else {
//                                             false
//                                         }
//                                     };

//                                     if should_stop {
//                                         break;
//                                     }

//                                     println!("up:{}  down:{}", up, down);
//                                     let mut data = market_data.lock().await;
//                                     data.update_prices(*up, *down);
//                                 }
//                             }
//                         }
//                     }
//                 }

//                 println!("⚠️ WS Disconnected. Reconnecting...");
//             }
//             Err(e) => {
//                 println!("❌ WS Connection failed: {:?}", e);
//             }
//         }

//         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
//     }
// }

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::models::market::MarketData;

pub async fn start_ws(
    up_id: String,
    down_id: String,
    market_data: Arc<Mutex<MarketData>>,
    expected_market_id: String,
) {
    // Single connection attempt - no outer loop
    println!("🔌 Connecting WebSocket for market: {}", expected_market_id);
    
    let url = Url::parse("wss://ws-subscriptions-clob.polymarket.com/ws/market").unwrap();
    
    match connect_async(url).await {
        Ok((mut ws, _)) => {
            println!("✅ WS Connected for market: {}", expected_market_id);
            
            let sub = json!({
                "assets_ids": [up_id, down_id],
                "type": "market"
            });
            
            if let Err(e) = ws.send(
                tokio_tungstenite::tungstenite::Message::Text(sub.to_string()),
            ).await {
                println!("❌ Failed to send subscription: {}", e);
                return;
            }
            
            let mut prices: HashMap<String, f64> = HashMap::new();
            
            while let Some(msg) = ws.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        println!("⚠️ WebSocket error: {}", e);
                        break;
                    }
                };
                
                if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                    let ws_data: serde_json::Value = match serde_json::from_str(&txt) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    
                    if ws_data["event_type"] == "last_trade_price" {
                        let asset = match ws_data["asset_id"].as_str() {
                            Some(a) => a,
                            None => continue,
                        };
                        
                        let price = match ws_data["price"].as_str() {
                            Some(p) => p.parse::<f64>().ok(),
                            None => None,
                        };
                        
                        if let Some(price) = price {
                            prices.insert(asset.to_string(), price);
                            
                            if let (Some(up), Some(down)) =
                                (prices.get(&up_id), prices.get(&down_id))
                            {
                                // Check if market should stop (without holding lock long)
                                let should_stop = {
                                    let data = market_data.lock().await;
                                    let now = crate::utils::time::now_ts();
                                    if now > data.end_ts + 5 {
                                        println!("🛑 WS stopping - market expired: {}", expected_market_id);
                                        true
                                    } else if data.market_id != expected_market_id {
                                        println!("🛑 WS stopping - market changed: {}", expected_market_id);
                                        true
                                    } else {
                                        false
                                    }
                                };
                                
                                if should_stop {
                                    return; // Exit completely, no reconnect
                                }
                                
                                // println!("up:{}  down:{}", up, down);
                                let mut data = market_data.lock().await;
                                data.update_prices(*up, *down);
                            }
                        }
                    }
                }
            }
            
            println!("⚠️ WS disconnected for market: {}", expected_market_id);
        }
        Err(e) => {
            println!("❌ WS Connection failed for market {}: {:?}", expected_market_id, e);
            // Only sleep briefly and return - don't reconnect
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    println!("🏁 WS task ending for market: {}", expected_market_id);
}