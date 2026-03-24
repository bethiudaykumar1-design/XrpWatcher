// use super::market::MarketData;

// #[derive(Debug)]
// pub struct MarketBuffer {
//     pub prev : Option<MarketData>,
//     pub curr : Option<MarketData>,
//     pub futu : Option<MarketData>,
// }



// impl MarketBuffer {
//     pub fn new()-> Self {
//         Self { 
//             prev:None,
//             curr:None,
//             futu:None,
//          }
//     }

//     // shifts market forward
//     pub fn shift(&mut self) {
//         println!("shifting markets");

//         self.prev = self.curr.take();
//         self.curr = self.futu.take();
//         self.futu = None
//     }
// }

use std::sync::Arc;
use tokio::sync::{Mutex};

use super::market::MarketData;

#[derive(Debug)]
pub struct MarketBuffer {
    pub prev: Option<Arc<Mutex<MarketData>>>,
    pub curr: Option<Arc<Mutex<MarketData>>>,
    pub futu: Option<Arc<Mutex<MarketData>>>,
    // pub shutdown_tx: Option<watch::Sender<bool>>,
}

impl MarketBuffer {
    pub fn new() -> Self {
        Self {
            prev: None,
            curr: None,
            futu: None,
            // shutdown_tx:None,
        }
    }

    pub fn shift(&mut self) {
        // println!("🔄 SHIFTING MARKETS");

        self.prev = self.curr.take();
        self.curr = self.futu.take();
        self.futu = None;
    }
}