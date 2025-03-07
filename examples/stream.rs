use futures_util::StreamExt;
use managed_order_book::{managed::ManagedBinanceOrderBook, quoter::Quoter, BinanceOrderBook};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = "RONINUSDT";

    let mut book_stream = ManagedBinanceOrderBook::new(symbol.to_string()).await?;

    while let Some(_) = book_stream.next().await {
        book_stream.book().print(10);
        let size = Decimal::from(100);
        let price = book_stream.book().buy(&size);
        dbg!(size, price);
        let size = Decimal::from(100);
        let price = book_stream.book().sell(&size);
        dbg!(size, price);
    }

    Ok(())
}
