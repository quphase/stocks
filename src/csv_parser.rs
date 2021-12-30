use chrono;
use serde::Deserialize;
use std::collections::HashMap;

pub type Trades = HashMap<String, Vec<Trade>>;

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub order_type: String,
    pub side: String,
    pub fees: f32,
    pub quantity: f64,
    pub average_price: f64,
}

pub fn parse(csv: &str) -> Result<Trades, csv::Error> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut trades = HashMap::new();
    for trade in reader.deserialize() {
        let trade: Trade = trade?;
        trades
            .entry(trade.symbol.clone())
            .or_insert(Vec::new())
            .push(trade);
    }
    Ok(trades)
}
