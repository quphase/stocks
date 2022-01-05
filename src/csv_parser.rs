use chrono;
use serde::Deserialize;
use std::collections::HashMap;

pub type Trades = HashMap<String, Vec<Trade>>;
pub type OptionTrades = HashMap<String, Vec<OptionTrade>>;

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub order_type: String,
    pub side: String,
    pub fees: f64,
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

#[derive(Debug, Clone, Deserialize)]
pub struct OptionTrade {
    pub chain_symbol: String,
    pub expiration_date: chrono::NaiveDate,
    pub strike_price: f64,
    pub option_type: String,
    pub side: String,
    pub order_created_at: chrono::DateTime<chrono::Utc>,
    pub direction: String,
    pub order_quantity: f64,
    pub order_type: String,
    pub opening_strategy: Option<String>,
    pub closing_strategy: Option<String>,
    pub price: f64,
    pub processed_quantity: f64,
}

pub fn parse_options(csv: &str) -> Result<OptionTrades, csv::Error> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut trades = HashMap::new();
    for trade in reader.deserialize() {
        let trade: OptionTrade = trade?;
        trades
            .entry(trade.chain_symbol.clone())
            .or_insert(Vec::new())
            .push(trade);
    }
    Ok(trades)
}
