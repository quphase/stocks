use crate::csv_parser::Trades;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Information {
    // Buy of certain quantity, price, and time
    Buy(f64, f64, chrono::DateTime<chrono::Utc>),
    // Sell of certain quantity, price, and time, and how many buys are covered
    Sell(f64, f64, chrono::DateTime<chrono::Utc>),
    // Profit between sell and latest buy
    PriceDiff(f64),
    // Time passic between sell and latest buy
    TimeDiff(chrono::Duration),
    // Total fee
    Fees(f64),
    // Remaining stocks left
    Remaing(f64),
    // A sell without a buy
    WeirdSell,
}

pub type AllInfo = HashMap<String, Vec<Information>>;

pub fn parse(trades: &Trades, symbol_filter: String) -> AllInfo {
    let mut result = AllInfo::new();
    for (symbol, data) in trades {
        if !symbol.contains(&symbol_filter) {
            continue;
        }

        let mut informations = Vec::new();

        let mut data = data.clone();
        data.sort_by(|a, b| a.date.partial_cmp(&b.date).unwrap());
        // A stack to track buy and sell orders such that the sell goes exhausts the latests buys
        // first
        let mut stack = Vec::new();
        for mut d in data {
            let side = &d.side;

            // we have a buy, so push it to into the stack
            if side == "buy" {
                informations.push(Information::Buy(
                    d.quantity,
                    d.average_price,
                    d.date.clone(),
                ));
                stack.push(d);
            } else {
                // we have a sell
                // keep poping from the stack until the quanity from the sell exhauts all the buy
                // quantities
                informations.push(Information::Sell(d.quantity, d.average_price, d.date));
                let process = || -> Option<()> {
                    let mut prev_d = stack.pop()?;
                    let mut quantity = d.quantity;

                    loop {
                        quantity = prev_d.quantity - quantity;

                        let time_diff = d.date - prev_d.date;
                        informations.push(Information::TimeDiff(time_diff));

                        let price_diff = (d.average_price - prev_d.average_price) * prev_d.quantity;
                        informations.push(Information::PriceDiff(price_diff));

                        informations.push(Information::Fees(d.quantity * d.fees));

                        // All the sell quantity have been exhausted
                        if quantity >= 0.0 {
                            break;
                        }
                        prev_d = stack.pop()?;
                        quantity = quantity.abs();
                    }

                    // leave the remaining amout of buys quantity in the stack
                    d.quantity = quantity;
                    if quantity > 0.0 {
                        stack.push(d);
                    }
                    Some(())
                };

                if process().is_none() {
                    informations.push(Information::WeirdSell);
                }

                //let res = process().unwrap_or("Selling more than owned?\n".to_string());
            }
        }
        let mut remaining = 0.;
        while let Some(last) = stack.pop() {
            remaining += last.quantity;
        }
        informations.push(Information::Remaing(remaining));
        //informations.push(Information::Remaing(stack.len() as f64));
        result
            .entry(symbol.to_string())
            .or_insert(Vec::new())
            .append(&mut informations);
    }
    result
}
