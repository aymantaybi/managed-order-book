use futures_util::StreamExt;
use managed_order_book::{quoter::Quoter, BinanceOrderBook, Depth, DepthUpdate, OrderBook};
use rust_decimal::Decimal;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = "CRVUSDC";

    let url = format!(
        "wss://fstream.binance.com/ws/{}@depth@100ms",
        symbol.to_lowercase()
    );

    let (mut ws_stream, _) = connect_async(url).await?;

    let depth: Depth = reqwest::get(format!(
        "https://fapi.binance.com/fapi/v1/depth?symbol={}&limit=1000",
        symbol.to_uppercase()
    ))
    .await?
    .json()
    .await?;

    let mut binance_order_book = BinanceOrderBook::new(symbol.to_string(), depth);

    dbg!(&binance_order_book);

    while let Some(item) = ws_stream.next().await {
        let message = item.unwrap();
        if let Message::Text(text) = message {
            let depth_update: DepthUpdate = serde_json::from_str(&text).unwrap();
            let _ = binance_order_book.update(depth_update);
            binance_order_book.print(10);
            let size = Decimal::from(100);
            let price = binance_order_book.buy(&size);
            dbg!(size, price);
            let size = Decimal::from(100);
            let price = binance_order_book.sell(&size);
            dbg!(size, price);
        }
    }

    Ok(())
}
