use prettytable::{row, table, Cell, Row, Table};
use quoter::Quoter;
use rust_decimal::{prelude::Zero, Decimal};
use serde::Deserialize;
use std::collections::BTreeMap;

pub mod managed;
pub mod quoter;

pub trait OrderBook {
    /// Returns the price decimal points
    fn price_decimal_points(&self) -> u32;
    /// Returns the quantity decimal points
    fn quantity_decimal_points(&self) -> u32;
    /// Returns the price precision
    fn tick_size(&self) -> Decimal;
    /// Returns the quantity precision
    fn step_size(&self) -> Decimal;
    //
    fn bids(&self) -> &BTreeMap<Decimal, Decimal>;
    fn asks(&self) -> &BTreeMap<Decimal, Decimal>;
    fn bids_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal>;
    fn asks_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal>;
    /// Returns (price, quantity)
    fn best_bid(&self) -> (&Decimal, &Decimal) {
        let bids = self.bids();
        bids.last_key_value().unwrap()
    }
    /// Returns (price, quantity)
    fn best_ask(&self) -> (&Decimal, &Decimal) {
        let asks = self.asks();
        asks.first_key_value().unwrap()
    }

    fn update_book(book: &mut BTreeMap<Decimal, Decimal>, update: Vec<(Decimal, Decimal)>) {
        for (price, quantity) in update {
            if quantity.is_zero() {
                book.remove(&price);
            } else {
                book.insert(price, quantity);
            }
        }
    }

    fn update_asks(&mut self, asks: Vec<(Decimal, Decimal)>) {
        let mut book = self.asks_mut();
        Self::update_book(&mut book, asks);
    }

    fn update_bids(&mut self, bids: Vec<(Decimal, Decimal)>) {
        let mut book = self.bids_mut();
        Self::update_book(&mut book, bids);
    }
}

#[derive(Debug)]
pub struct BinanceOrderBook {
    symbol: String,
    last_update_id: u128,
    processed_events_count: u128,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl BinanceOrderBook {
    pub fn new(symbol: String, depth: Depth) -> Self {
        let Depth {
            bids,
            asks,
            last_update_id,
            ..
        } = depth;
        let bids = BTreeMap::from_iter(bids.into_iter());
        let asks = BTreeMap::from_iter(asks.into_iter());
        Self {
            symbol,
            bids,
            asks,
            last_update_id,
            processed_events_count: 0,
        }
    }

    pub fn update(&mut self, depth_update: DepthUpdate) -> anyhow::Result<()> {
        let DepthUpdate {
            event_type,
            event_time,
            transaction_time,
            symbol,
            first_update_id,
            final_update_id,
            last_stream_final_update_id,
            bids,
            asks,
        } = depth_update;

        if self.processed_events_count == 0 {
            if self.last_update_id < first_update_id || final_update_id < self.last_update_id {
                return Err(anyhow::anyhow!("First event out of sync"));
            }
        } else {
            if self.last_update_id != last_stream_final_update_id {
                return Err(anyhow::anyhow!("Event out of sync"));
            }
        }

        self.update_asks(asks);
        self.update_bids(bids);

        self.last_update_id = final_update_id;
        self.processed_events_count += 1;

        Ok(())
    }

    pub fn print(&self, levels: usize) {
        let mut bid_table = Table::new();

        bid_table.add_row(row!["Size", "Price"]);

        for (price, quantity) in self.bids().iter().rev().take(levels) {
            bid_table.add_row(Row::new(vec![
                Cell::new(&quantity.to_string()),
                Cell::new(&price.to_string()),
            ]));
        }

        let mut ask_table = Table::new();

        ask_table.add_row(row!["Price", "Size"]);

        for (price, quantity) in self.asks().iter().take(levels) {
            ask_table.add_row(Row::new(vec![
                Cell::new(&price.to_string()),
                Cell::new(&quantity.to_string()),
            ]));
        }

        let table = table!(["Bids", "Asks"], [bid_table, ask_table]);

        table.printstd();
    }
}

impl OrderBook for BinanceOrderBook {
    fn price_decimal_points(&self) -> u32 {
        todo!()
    }

    fn quantity_decimal_points(&self) -> u32 {
        todo!()
    }

    fn tick_size(&self) -> Decimal {
        todo!()
    }

    fn step_size(&self) -> Decimal {
        todo!()
    }

    fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    fn bids_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.bids
    }

    fn asks_mut(&mut self) -> &mut BTreeMap<Decimal, Decimal> {
        &mut self.asks
    }
}

impl Quoter for BinanceOrderBook {
    fn buy(&self, size: &Decimal) -> Decimal {
        let book = self.asks().iter();
        <BinanceOrderBook as Quoter>::quote(size, book)
    }

    fn sell(&self, size: &Decimal) -> Decimal {
        let book = self.bids().iter().rev();
        <BinanceOrderBook as Quoter>::quote(size, book)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    last_update_id: u128,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "T")]
    transaction_time: u64,
    bids: Vec<(Decimal, Decimal)>,
    asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthUpdate {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "T")]
    transaction_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "U")]
    first_update_id: u128,
    #[serde(rename = "u")]
    final_update_id: u128,
    #[serde(rename = "pu")]
    last_stream_final_update_id: u128,
    #[serde(rename = "b")]
    bids: Vec<(Decimal, Decimal)>,
    #[serde(rename = "a")]
    asks: Vec<(Decimal, Decimal)>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let symbol = "RONINUSDT";
        let data = r#"{"lastUpdateId":5573970560080,"E":1729534992908,"T":1729534992900,"bids":[["1.663900","31.0"],["1.663800","71.3"],["1.663700","30.6"],["1.663600","110.2"],["1.663500","76.7"]],"asks":[["1.664500","40.3"],["1.664700","10.0"],["1.664800","133.9"],["1.664900","214.9"],["1.665000","110.2"]]}"#;
        let depth: Depth = serde_json::from_str(&data).unwrap();
        let mut binance_order_book = BinanceOrderBook::new(symbol.to_string(), depth);
        let best_bid = binance_order_book.best_bid();
        let best_ask = binance_order_book.best_ask();
        // dbg!(best_bid, best_ask);
        // First update
        let data = r#"{"e":"depthUpdate","E":1729534992913,"T":1729534992900,"s":"RONINUSDT","U":5573970557923,"u":5573970560080,"pu":5573970557478,"b":[["1.331200","45.0"],["1.658900","1013.1"]],"a":[["1.669200","2277.2"],["1.670000","783.4"],["1.670100","1136.5"],["1.670900","739.9"],["1.675100","1103.4"],["1.676600","120.6"],["1.677800","2083.7"]]}"#;
        let depth_update: DepthUpdate = serde_json::from_str(&data).unwrap();
        // dbg!(&depth_update);
        let result = binance_order_book.update(depth_update);
        dbg!(result);
        // Second update
        let data = r#"{"e":"depthUpdate","E":1729534993015,"T":1729534993006,"s":"RONINUSDT","U":5573970560466,"u":5573970562617,"pu":5573970560080,"b":[["1.331200","0.0"],["1.655500","372.0"],["1.660700","2010.4"],["1.660800","2022.0"],["1.660900","759.9"],["1.662000","698.6"],["1.662200","139.2"],["1.662900","448.4"],["1.663000","143.0"],["1.663700","40.6"],["1.664000","30.3"]],"a":[["1.665500","72.5"],["1.666000","47.3"],["1.667700","795.3"],["1.669200","2081.3"],["1.669300","203.8"],["1.669600","757.7"],["1.670000","298.1"],["1.670800","1890.1"]]}"#;
        let depth_update: DepthUpdate = serde_json::from_str(&data).unwrap();
        //  dbg!(&depth_update);
        let result = binance_order_book.update(depth_update);
        dbg!(result);
        dbg!(binance_order_book);
    }
}
