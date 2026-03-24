use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy::dotenv;
use std::env;

mod utils;
mod api;
mod models;
mod db;

use utils::time::*;
use api::gamma::*;
use api::clob::fetch_initial_price;
use api::resolve::resolve_result;
use api::ws::start_ws;

use models::market::MarketData;
use models::buffer::MarketBuffer;
use db::Database;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();
    
    println!("🚀 xrp Polymarket Watcher (WS MODE)");
    
    // Initialize database
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let db = match Database::new(&database_url).await {
        Ok(db) => {
            println!("✅ Database connected");
            db
        }
        Err(e) => {
            eprintln!("❌ Database connection failed: {}", e);
            std::process::exit(1);
        }
    };
    
    let db = Arc::new(db);
    
    let mut buffer = MarketBuffer::new();

    loop {
        let now = now_ts();
        let window_ts = current_window_ts(now);
        let slug = generate_slug(window_ts);
        let end_ts = window_end_ts(window_ts);

        // 🟢 INIT CURRENT MARKET
        if buffer.curr.is_none() {
            // println!("\n🚀 NEW CYCLE: {}", slug);

            let market = match fetch_with_retry(&slug).await {
                Some(m) => m,
                None => {
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            let (up_token, down_token) = match extract_token_ids(&market) {
                Some(tokens) => tokens,
                None => continue,
            };

            // Fetch initial prices
            // println!("📊 Fetching initial prices...");
            
            let initial_up_price = fetch_initial_price(&up_token).await;
            let initial_down_price = fetch_initial_price(&down_token).await;
            
            // println!("📈 INITIAL PRICES - UP: {:.4}, DOWN: {:.4}", initial_up_price, initial_down_price);

            let market_id = market.id.clone();
            let title = market.question.clone();

            let market_data = MarketData::new(
                market_id.clone(),
                slug.clone(),
                title.clone(),
                window_ts,
                end_ts,
                up_token.clone(),
                down_token.clone(),
                initial_up_price,
                initial_down_price,
            );

            let shared = Arc::new(Mutex::new(market_data));

            tokio::spawn(start_ws(
                up_token.clone(),
                down_token.clone(),
                shared.clone(),
                market_id,
            ));

            buffer.curr = Some(shared);

            // println!("✅ WS started for current market");
        }

        // 🔁 TRACK CURRENT MARKET
        if let Some(curr) = buffer.curr.as_ref() {
            let (end_ts, low_up, low_down, market_id, title, start_ts, initial_up, initial_down, last_up, last_down) = {
                let data = curr.lock().await;
                (
                    data.end_ts,
                    data.low_up_price,
                    data.low_down_price,
                    data.market_id.clone(),
                    data.title.clone(),
                    data.start_ts,
                    data.start_up_price,
                    data.start_down_price,
                    data.last_up_price,
                    data.last_down_price,
                )
            };

            let now = now_ts();
            let secs_left = end_ts - now;

            // println!(
            //     "⏱ {} sec left | LOW UP: {:.4} | LOW DOWN: {:.4}",
            //     secs_left, low_up, low_down
            // );

            // 🔮 PRELOAD NEXT MARKET
            if secs_left <= 20 && buffer.futu.is_none() {
                // println!("🔮 Preloading next market...");

                let next_window = end_ts;
                let next_slug = generate_slug(next_window);

                if let Some(next_market) = fetch_with_retry(&next_slug).await {
                    if let Some((up_token, down_token)) = extract_token_ids(&next_market) {
                        let next_market_id = next_market.id.clone();
                        let next_title = next_market.question.clone();
                        
                        // Fetch initial prices for next market
                        // println!("📊 Fetching initial prices for next market...");
                        
                        let next_up_price = fetch_initial_price(&up_token).await;
                        let next_down_price = fetch_initial_price(&down_token).await;
                        
                        // println!("📈 NEXT MARKET INITIAL - UP: {:.4}, DOWN: {:.4}", next_up_price, next_down_price);

                        let futu_data = MarketData::new(
                            next_market_id.clone(),
                            next_slug,
                            next_title,
                            next_window,
                            next_window + 300,
                            up_token.clone(),
                            down_token.clone(),
                            next_up_price,
                            next_down_price,
                        );

                        let futu_shared = Arc::new(Mutex::new(futu_data));

                        tokio::spawn(start_ws(
                            up_token,
                            down_token,
                            futu_shared.clone(),
                            next_market_id,
                        ));

                        buffer.futu = Some(futu_shared);

                        // println!("✅ Next market WS ready");
                    }
                }
            }

            // 🔄 MARKET END
            if secs_left <= 0 {
                // println!("🔄 shifting markets");

                buffer.shift();

                if let Some(prev) = buffer.prev.as_ref() {
                    let prev_data = prev.lock().await;

                    // println!("\n📊 MARKET COMPLETED");
                    // println!("Market ID: {}", prev_data.market_id);
                    // println!("Title: {}", prev_data.title);
                    // println!("📈 INITIAL PRICES:");
                    // println!("UP:   {:.4}", prev_data.start_up_price);
                    // println!("DOWN: {:.4}", prev_data.start_down_price);
                    // println!("📉 LOWEST PRICES:");
                    // println!("LOW UP:   {:.4}", prev_data.low_up_price);
                    // println!("LOW DOWN: {:.4}", prev_data.low_down_price);

                    let result = resolve_result(
                        &prev_data.market_id,
                        prev_data.last_up_price,
                        prev_data.last_down_price,
                    )
                    .await;

                    // println!("✅ RESULT: {}", result);
                    
                    // 💾 SAVE TO DATABASE
                    if let Err(e) = db.save_market_result(
                        &prev_data.title,
                        prev_data.start_ts,
                        prev_data.end_ts,
                        prev_data.start_up_price,
                        prev_data.start_down_price,
                        prev_data.low_up_price,
                        prev_data.low_down_price,
                        prev_data.last_up_price,
                        prev_data.last_down_price,
                        &result,
                    ).await {
                        eprintln!("❌ Failed to save to database: {}", e);
                    } else {
                        // println!("💾 Saved to database");
                    }
                    
                    // println!("==============================\n");
                }
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}