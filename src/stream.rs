use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{Stream, StreamExt};
use rust_decimal::{prelude::Zero, Decimal};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{quoter::Quoter, BinanceOrderBook, Depth, DepthUpdate};

pub struct OrderBookStream {
    pub binance_order_book: BinanceOrderBook,
    pub web_socket_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl OrderBookStream {
    pub async fn new(symbol: String) -> anyhow::Result<Self> {
        let url = format!(
            "wss://fstream.binance.com/ws/{}@depth@100ms",
            symbol.to_lowercase()
        );

        let (web_socket_stream, _) = connect_async(url).await?;

        let depth: Depth = reqwest::get(format!(
            "https://fapi.binance.com/fapi/v1/depth?symbol={}&limit=1000",
            symbol.to_uppercase()
        ))
        .await?
        .json()
        .await?;

        let binance_order_book = BinanceOrderBook::new(symbol.to_string(), depth);

        Ok(Self {
            binance_order_book,
            web_socket_stream,
        })
    }

    pub fn book(&self) -> &BinanceOrderBook {
        &self.binance_order_book
    }
}

impl Stream for OrderBookStream {
    type Item = u64;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut next = false;

        while let Poll::Ready(item) = self.web_socket_stream.poll_next_unpin(cx) {
            let message = item.unwrap().unwrap(); // todo
            if let Message::Text(text) = message {
                let depth_update: DepthUpdate = serde_json::from_str(&text).unwrap();
                let _ = self.binance_order_book.update(depth_update);
                next = true;
            }
        }

        if next {
            return Poll::Ready(Some(69));
        }

        Poll::Pending
    }
}
