use crate::csv_parser::OptionTrades;
use std::collections::HashMap;

pub enum Information {
    BuyToOpen(f64),
    SellToOpen(f64),
    BuyToClose(f64),
    SellToClose(f64),
    TotalDiff(f64),
}

pub type AllOptionInfo = HashMap<String, Vec<Information>>;

pub fn parse(
    trades: &OptionTrades,
    symbol_filter: String,
    year: Option<chrono::DateTime<chrono::Utc>>,
) -> AllOptionInfo {
    let mut result = AllOptionInfo::new();
    for (symbol, data) in trades {
        if !symbol.contains(&symbol_filter) {
            continue;
        }

        let mut informations = Vec::new();

        let mut data = data.clone();
        data.sort_by(|a, b| a.order_created_at.partial_cmp(&b.order_created_at).unwrap());

        let mut stack = Vec::new();

        for mut d in &data {
            if let Some(opening_strategy) = &d.opening_strategy {
                stack.push(d);

                let mut print = || {
                    if d.side == "buy" {
                        informations.push(Information::BuyToOpen(-d.price * 100.));
                    } else {
                        informations.push(Information::SellToOpen(d.price * 100.));
                    }
                };
                if let Some(year) = year {
                    if d.order_created_at > year {
                        print();
                    }
                } else {
                    print();
                }
            }
            if let Some(closing_strategy) = &d.closing_strategy {
                stack.pop();
                let mut print = || {
                    if d.side == "buy" {
                        informations.push(Information::BuyToClose(-d.price * 100.));
                    } else {
                        informations.push(Information::SellToClose(d.price * 100.));
                    }
                };
                if let Some(year) = year {
                    if d.order_created_at > year {
                        print();
                    }
                } else {
                    print();
                }
            }
        }
        let mut total_diff = 0.;
        for info in &informations {
            match info {
                Information::BuyToOpen(p)
                | Information::SellToOpen(p)
                | Information::BuyToClose(p)
                | Information::SellToClose(p) => total_diff += p,
                _ => {}
            }
        }
        informations.push(Information::TotalDiff(total_diff));
        if informations.len() > 0 {
            result
                .entry(symbol.to_string())
                .or_insert(Vec::new())
                .append(&mut informations);
        }
    }

    result
}
