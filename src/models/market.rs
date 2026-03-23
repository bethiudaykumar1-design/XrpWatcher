#[derive(Debug, Clone)]
pub struct MarketData {
    pub market_id: String,
    pub slug: String,
    pub title: String,  // Add this
    pub start_ts: i64,
    pub end_ts: i64,
    pub up_token_id: String,
    pub down_token_id: String,
    pub start_up_price: f64,
    pub start_down_price: f64,
    pub low_up_price: f64,
    pub low_down_price: f64,
    pub last_up_price: f64,
    pub last_down_price: f64,
    pub result: Option<String>,
}

impl MarketData {
    /// Initialize new market
    pub fn new(
        market_id: String,
        slug: String,
        title: String,  // Add title parameter
        start_ts: i64,
        end_ts: i64,
        up_token_id: String,
        down_token_id: String,
        start_up_price: f64,
        start_down_price: f64,
    ) -> Self {
        Self {
            market_id,
            slug,
            title,  // Store title
            start_ts,
            end_ts,
            up_token_id,
            down_token_id,
            start_up_price,
            start_down_price,
            low_up_price: start_up_price,
            low_down_price: start_down_price,
            last_up_price: start_up_price,
            last_down_price: start_down_price,
            result: None,
        }
    }
    /// Update prices + track lows
    pub fn update_prices(&mut self, up: f64, down: f64) {
    self.last_up_price = up;
    self.last_down_price = down;

    // 🔥 FIRST TIME INIT
    if self.low_up_price == 0.0 {
        self.low_up_price = up;
    }
    if self.low_down_price == 0.0 {
        self.low_down_price = down;
    }

    if up < self.low_up_price {
        self.low_up_price = up;
        // println!("📉 New LOW UP: {:.4}", up);
    }

    if down < self.low_down_price {
        self.low_down_price = down;
        // println!("📉 New LOW DOWN: {:.4}", down);
    }
}
}