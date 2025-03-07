use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{Stream, StreamExt};
use rust_decimal::{prelude::Zero, Decimal};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{quoter::Quoter, BinanceOrderBook, Depth, DepthUpdate};

pub struct ManagedBinanceOrderBook {
    pub binance_order_book: BinanceOrderBook,
    pub web_socket_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub updated: bool,
}

impl ManagedBinanceOrderBook {
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
            updated: false,
        })
    }

    pub fn book(&self) -> &BinanceOrderBook {
        &self.binance_order_book
    }
}

impl Stream for ManagedBinanceOrderBook {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.updated = false;

        while let Poll::Ready(item) = this.web_socket_stream.poll_next_unpin(cx) {
            let message = item.unwrap().unwrap(); // todo
            if let Message::Text(text) = message {
                let depth_update: DepthUpdate = serde_json::from_str(&text).unwrap();
                let _ = this.binance_order_book.update(depth_update);
                this.updated = true;
            }
        }

        if this.updated {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}
