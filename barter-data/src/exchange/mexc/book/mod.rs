use crate::{
    books::Level,
    error::DataError,
    event::{MarketEvent, MarketIter},
    subscription::book::OrderBookL1,
};
use barter_instrument::exchange::ExchangeId;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use super::trade::proto;

fn ms_epoch_to_datetime_utc(ms: i64) -> Result<DateTime<Utc>, DataError> {
    if ms < 0 {
        return Err(DataError::Socket(format!(
            "Unsupported MexcBookTicker::Timestamp: invalid unix_epoch_ms (negative): {}",
            ms
        )));
    }
    DateTime::from_timestamp_millis(ms).ok_or_else(|| {
        DataError::Socket(format!(
            "Unsupported MexcBookTicker::Timestamp: invalid unix_epoch_ms: {}",
            ms
        ))
    })
}

fn parse_level(price: &str, qty: &str) -> Result<Level, DataError> {
    let price = price.parse::<Decimal>().map_err(|e| {
        DataError::Socket(format!(
            "Failed to parse price from MEXC agg book ticker: '{}', error: {}",
            price, e
        ))
    })?;
    let amount = qty.parse::<Decimal>().map_err(|e| {
        DataError::Socket(format!(
            "Failed to parse quantity from MEXC agg book ticker: '{}', error: {}",
            qty, e
        ))
    })?;
    Ok(Level::new(price, amount))
}

impl<InstrumentKey> From<(ExchangeId, InstrumentKey, proto::PushDataV3ApiWrapper)>
    for MarketIter<InstrumentKey, OrderBookL1>
where
    InstrumentKey: Clone,
{
    fn from(
        (exchange_id, instrument, wrapper): (
            ExchangeId,
            InstrumentKey,
            proto::PushDataV3ApiWrapper,
        ),
    ) -> Self {
        let time_received = Utc::now();
        if let Some(proto::push_data_v3_api_wrapper::Body::PublicAggreBookTicker(ticker)) =
            wrapper.body
        {
            let exchange_time = wrapper
                .send_time
                .or(wrapper.create_time)
                .and_then(|ms| ms_epoch_to_datetime_utc(ms).ok())
                .unwrap_or(time_received);

            let best_bid = match parse_level(&ticker.bid_price, &ticker.bid_quantity) {
                Ok(lvl) => Some(lvl),
                Err(err) => return Self(vec![Err(err)]),
            };
            let best_ask = match parse_level(&ticker.ask_price, &ticker.ask_quantity) {
                Ok(lvl) => Some(lvl),
                Err(err) => return Self(vec![Err(err)]),
            };

            return Self(vec![Ok(MarketEvent {
                time_exchange: exchange_time,
                time_received,
                exchange: exchange_id,
                instrument,
                kind: OrderBookL1 {
                    last_update_time: exchange_time,
                    best_bid,
                    best_ask,
                },
            })]);
        }
        Self(vec![])
    }
}
