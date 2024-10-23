use futures_util::StreamExt;
use orderbook_quoter::stream::OrderBookStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = "CRVUSDC";

    let mut s = OrderBookStream::new(symbol.to_string()).await?;

    while let Some(item) = s.next().await {
        s.book().print(2);
    }

    Ok(())
}
