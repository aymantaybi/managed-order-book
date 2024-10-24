use orderbook_quoter::{managed::ManagedBinanceOrderBook, BinanceOrderBook};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = "CRVUSDC";

    let callback = |book: &BinanceOrderBook| book.print(5);

    let m = ManagedBinanceOrderBook::new(symbol.to_string(), callback).await?;

    m.await;

    Ok(())
}
