use managed_order_book::{managed::ManagedBinanceOrderBook, BinanceOrderBook};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = "CRVUSDC";

    let m = ManagedBinanceOrderBook::new(symbol.to_string()).await?;

    m.await;

    Ok(())
}
