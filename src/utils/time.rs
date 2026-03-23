use chrono::Utc;

pub fn now_ts() ->i64 {
    Utc::now().timestamp()
}

// Returns start of current 5-min window
pub fn current_window_ts(now:i64) -> i64 {
    now-(now%300)
}

pub fn generate_slug(window_ts:i64) -> String {
    format!("xrp-updown-5m-{}",window_ts)
} 

pub fn window_end_ts(window_ts:i64) -> i64 {
    window_ts + 300
}