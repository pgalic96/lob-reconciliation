pub mod operations {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum UpdateTypeEnum {
        New,
        Change,
        Delete,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PriceUpdate {
        pub update_type: UpdateTypeEnum,
        pub price: f64,
        pub amount: f64,
    }

    pub fn update_bids(
        bid_order_book: &mut HashMap<String, f64>,
        bids: Vec<PriceUpdate>,
        max_bid: f64,
    ) -> f64 {
        let mut max_bid = max_bid;
        for bid in &bids {
            match bid.update_type {
                UpdateTypeEnum::Change => {
                    bid_order_book.insert(bid.price.to_string(), bid.amount);
                }
                UpdateTypeEnum::New => {
                    if bid.price > max_bid {
                        max_bid = bid.price;
                    }
                    bid_order_book.insert(bid.price.to_string(), bid.amount);
                }
                UpdateTypeEnum::Delete => {
                    bid_order_book.remove(&bid.price.to_string());
                    if bid.price == max_bid {
                        max_bid = compute_max_bid(bid_order_book);
                    }
                }
            }
        }
        max_bid
    }

    pub fn update_asks(
        ask_order_book: &mut HashMap<String, f64>,
        asks: Vec<PriceUpdate>,
        min_ask: f64,
    ) -> f64 {
        let mut min_ask = min_ask;
        for ask in &asks {
            match ask.update_type {
                UpdateTypeEnum::Change => {
                    ask_order_book.insert(ask.price.to_string(), ask.amount);
                }
                UpdateTypeEnum::New => {
                    if ask.price < min_ask {
                        min_ask = ask.price;
                    }
                    ask_order_book.insert(ask.price.to_string(), ask.amount);
                }
                UpdateTypeEnum::Delete => {
                    ask_order_book.remove(&ask.price.to_string());
                    if ask.price == min_ask {
                        min_ask = compute_min_ask(ask_order_book);
                    }
                }
            }
        }
        min_ask
    }

    pub fn init_bids(bid_order_book: &mut HashMap<String, f64>, bids: Vec<PriceUpdate>) -> f64 {
        let mut max_bid = 0.0;
        for bid in &bids {
            if bid.price > max_bid {
                max_bid = bid.price;
            }
            bid_order_book.insert(bid.price.to_string(), bid.amount);
        }
        max_bid
    }

    pub fn init_asks(ask_order_book: &mut HashMap<String, f64>, asks: Vec<PriceUpdate>) -> f64 {
        let mut min_ask = f64::MAX;
        for ask in &asks {
            if ask.price < min_ask {
                min_ask = ask.price;
            }
            ask_order_book.insert(ask.price.to_string(), ask.amount);
        }
        min_ask
    }

    fn compute_max_bid(bid_book: &HashMap<String, f64>) -> f64 {
        let mut max_bid: f64 = 0.0;
        for key in bid_book.keys() {
            let key: f64 = key.trim().parse().unwrap();
            if key > max_bid {
                max_bid = key;
            }
        }
        max_bid
    }

    fn compute_min_ask(ask_book: &HashMap<String, f64>) -> f64 {
        let mut min_ask: f64 = f64::MAX;
        for key in ask_book.keys() {
            let key: f64 = key.trim().parse().unwrap();
            if key < min_ask {
                min_ask = key;
            }
        }
        min_ask
    }
}
