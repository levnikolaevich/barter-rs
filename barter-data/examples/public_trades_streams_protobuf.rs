use barter_data::{
    event::DataKind, exchange::mexc::Mexc, streams::{consumer::MarketStreamResult, reconnect::stream::ReconnectingStream, Streams}, subscription::{book::OrderBooksL1, trade::PublicTrades}
};
use barter_instrument::instrument::market_data::{kind::MarketDataInstrumentKind, MarketDataInstrument};
use futures_util::StreamExt;
use tracing::{info, warn};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    init_logging();

    let streams: Streams<MarketStreamResult<MarketDataInstrument, DataKind>> =
        Streams::builder_multi()
        .add(
            Streams::<PublicTrades>::builder().subscribe([
                (Mexc, "eth", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
                (Mexc, "btc", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
            ]),
        )
        .add(
            Streams::<OrderBooksL1>::builder().subscribe([
                (Mexc, "eth", "usdt", MarketDataInstrumentKind::Spot, OrderBooksL1),
                (Mexc, "btc", "usdt", MarketDataInstrumentKind::Spot, OrderBooksL1),
            ]),
        )
        .init()
        .await
        .unwrap();

    let mut joined_stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    while let Some(event) = joined_stream.next().await {
        info!("{event:?}");
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_ansi(cfg!(debug_assertions))
        .json()
        .init();
}
