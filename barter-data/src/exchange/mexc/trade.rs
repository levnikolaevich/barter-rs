use barter_instrument::{exchange::ExchangeId, Side};
use chrono::{DateTime, Utc};

use crate::{
    event::{MarketEvent, MarketIter},
    error::DataError, // Import DataError
    subscription::trade::PublicTrade,
};

// Protobuf generated structs module
pub mod proto {
    #![allow(clippy::all)]
    #![allow(warnings)]
    include!("protobuf_gen/_.rs");
}

/// Converts MEXC `trade_type` (i32) to Barter `Side`.
/// - 1: Buy
/// - 2: Sell
fn mexc_trade_type_to_side(trade_type: i32) -> Result<Side, DataError> {
    match trade_type {
        1 => Ok(Side::Buy),
        2 => Ok(Side::Sell),
        _ => Err(DataError::Socket(format!( // Expects String for DataError::Socket
            "Unsupported MexcTrade::Side: unknown trade_type: {}",
            trade_type
        ))),
    }
}

/// Converts a millisecond Unix epoch timestamp (i64) to `DateTime<Utc>`.
fn ms_epoch_to_datetime_utc(ms: i64) -> Result<DateTime<Utc>, DataError> {
    if ms < 0 { // Added check for negative timestamps
        return Err(DataError::Socket(format!(
            "Unsupported MexcTrade::Timestamp: invalid unix_epoch_ms (negative): {}",
            ms
        )));
    }
    DateTime::from_timestamp_millis(ms).ok_or_else(|| {
        DataError::Socket(format!( 
            "Unsupported MexcTrade::Timestamp: invalid unix_epoch_ms: {}",
            ms
        ))
    })
}

impl<InstrumentKey> From<(ExchangeId, InstrumentKey, proto::PushDataV3ApiWrapper)>
    for MarketIter<InstrumentKey, PublicTrade>
where
    InstrumentKey: Clone,
{
    fn from(
        (exchange_id, instrument, wrapper): (ExchangeId, InstrumentKey, proto::PushDataV3ApiWrapper),
    ) -> Self {
        let mut market_events = Vec::new();
        let time_received = Utc::now();

        if let Some(body) = wrapper.body {
            match body {
                proto::push_data_v3_api_wrapper::Body::PublicDeals(deals_api) => {
                    for deal_item in deals_api.deals {
                        let result = map_public_deals_item_to_market_event(
                            exchange_id,
                            instrument.clone(),
                            &deal_item,
                            time_received,
                        );
                        market_events.push(result);
                    }
                }
                proto::push_data_v3_api_wrapper::Body::PublicAggreDeals(aggre_deals_api) => {
                    for deal_item in aggre_deals_api.deals {
                         let result = map_public_aggre_deals_item_to_market_event(
                            exchange_id,
                            instrument.clone(),
                            &deal_item,
                            time_received,
                        );
                        market_events.push(result);
                    }
                }
                _ => {} // Other message types not handled here
            }
        }
        MarketIter(market_events) // MarketIter expects Vec<Result<_, DataError>>
    }
}

// Helper function to map proto::PublicDealsV3ApiItem to MarketEvent<PublicTrade>
fn map_public_deals_item_to_market_event<InstrumentKey: Clone>(
    exchange_id: ExchangeId,
    instrument: InstrumentKey,
    deal_item: &proto::PublicDealsV3ApiItem,
    time_received: DateTime<Utc>,
) -> Result<MarketEvent<InstrumentKey, PublicTrade>, DataError> {
    let price = deal_item.price.parse::<f64>().map_err(|e| {
        DataError::Socket(format!( 
            "Failed to parse price from MEXC deal: '{}', error: {}",
            deal_item.price, e
        ))
    })?;
    let amount = deal_item.quantity.parse::<f64>().map_err(|e| {
        DataError::Socket(format!( 
            "Failed to parse quantity from MEXC deal: '{}', error: {}",
            deal_item.quantity, e
        ))
    })?;
    let side = mexc_trade_type_to_side(deal_item.trade_type)?;
    let exchange_time = ms_epoch_to_datetime_utc(deal_item.time)?;

    Ok(MarketEvent {
        time_exchange: exchange_time,
        time_received,
        exchange: exchange_id,
        instrument,
        kind: PublicTrade {
            id: deal_item.time.to_string(),
            price,
            amount,
            side,
        },
    })
}

// Helper function to map proto::PublicAggreDealsV3ApiItem to MarketEvent<PublicTrade>
fn map_public_aggre_deals_item_to_market_event<InstrumentKey: Clone>(
    exchange_id: ExchangeId,
    instrument: InstrumentKey,
    deal_item: &proto::PublicAggreDealsV3ApiItem,
    time_received: DateTime<Utc>,
) -> Result<MarketEvent<InstrumentKey, PublicTrade>, DataError> {
    let price = deal_item.price.parse::<f64>().map_err(|e| {
        DataError::Socket(format!( 
            "Failed to parse price from MEXC aggregated deal: '{}', error: {}",
            deal_item.price, e
        ))
    })?;
    let amount = deal_item.quantity.parse::<f64>().map_err(|e| {
        DataError::Socket(format!( 
            "Failed to parse quantity from MEXC aggregated deal: '{}', error: {}",
            deal_item.quantity, e
        ))
    })?;
    let side = mexc_trade_type_to_side(deal_item.trade_type)?;
    let exchange_time = ms_epoch_to_datetime_utc(deal_item.time)?;

    Ok(MarketEvent {
        time_exchange: exchange_time,
        time_received,
        exchange: exchange_id,
        instrument,
        kind: PublicTrade {
            id: deal_item.time.to_string(),
            price,
            amount,
            side,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Identifier;
    use serde::{Serialize, Deserialize};
    use std::time::Duration;
    use barter_integration::de::datetime_utc_from_epoch_duration;

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct TestInstrument {
        base: String,
        quote: String,
    }

    impl Identifier<String> for TestInstrument {
        fn id(&self) -> String {
            format!("{}_{}", self.base, self.quote)
        }
    }

    #[test]
    fn test_mexc_trade_type_to_side_conversion() {
        assert_eq!(mexc_trade_type_to_side(1), Ok(Side::Buy));
        assert_eq!(mexc_trade_type_to_side(2), Ok(Side::Sell));
        match mexc_trade_type_to_side(0) {
            Err(DataError::Socket(s)) => { 
                assert!(s.contains("Unsupported MexcTrade::Side"));
                assert!(s.contains("unknown trade_type: 0"));
            }
            other => panic!("Expected DataError::Socket(String), got {:?}", other),
        }
    }

    #[test]
    fn test_ms_epoch_to_datetime_utc_conversion() {
        let timestamp_ms_valid = 1609459200000i64;
        let expected_datetime = datetime_utc_from_epoch_duration(Duration::from_millis(timestamp_ms_valid as u64));
        assert_eq!(ms_epoch_to_datetime_utc(timestamp_ms_valid), Ok(expected_datetime));
        
        let timestamp_ms_invalid = -1i64;
        match ms_epoch_to_datetime_utc(timestamp_ms_invalid) { // Test with -1
             Err(DataError::Socket(s)) => { 
                assert!(s.contains("Unsupported MexcTrade::Timestamp"));
                assert!(s.contains("invalid unix_epoch_ms (negative): -1")); // Updated error message check
            }
            other => panic!("Expected DataError::Socket(String) for negative timestamp, got {:?}", other),
        }

        // Test with a large value that might fail parsing if not handled by from_timestamp_millis
        // but should be caught by our negative check if it were negative.
        // This specific test case for from_timestamp_millis failing is harder to hit without
        // knowing its exact internal limits for i64 if they are less than i64::MAX.
        // For now, the negative check is the primary concern from the failed test.
    }

    #[test]
    fn test_public_aggre_deals_item_to_market_event() {
        let deal_item = proto::PublicAggreDealsV3ApiItem {
            price: "50000.50".to_string(),
            quantity: "0.001".to_string(),
            trade_type: 1, // Buy
            time: 1609459200123,
        };

        let instrument = TestInstrument { base: "BTC".into(), quote: "USDT".into() };
        let time_received = Utc::now();

        let event = map_public_aggre_deals_item_to_market_event(
            ExchangeId::Mexc,
            instrument.clone(),
            &deal_item,
            time_received,
        ).unwrap();

        assert_eq!(event.exchange, ExchangeId::Mexc);
        assert_eq!(event.instrument.id(), "BTC_USDT".to_string());
        assert_eq!(event.kind.price, 50000.50);

        // Test parsing failure for price
        let deal_item_bad_price = proto::PublicAggreDealsV3ApiItem {
            price: "not_a_float".to_string(),
            quantity: "0.001".to_string(),
            trade_type: 1,
            time: 1609459200123,
        };
        let result_bad_price = map_public_aggre_deals_item_to_market_event(
            ExchangeId::Mexc,
            instrument.clone(),
            &deal_item_bad_price,
            time_received,
        );
        assert!(matches!(result_bad_price, Err(DataError::Socket(_)))); 
        if let Err(DataError::Socket(s)) = result_bad_price {
            assert!(s.contains("Failed to parse price"));
        }

        // Test parsing failure for quantity
        let deal_item_bad_quantity = proto::PublicAggreDealsV3ApiItem {
            price: "50000.50".to_string(),
            quantity: "not_a_float".to_string(),
            trade_type: 1,
            time: 1609459200123,
        };
        let result_bad_quantity = map_public_aggre_deals_item_to_market_event(
            ExchangeId::Mexc,
            instrument.clone(),
            &deal_item_bad_quantity,
            time_received,
        );
        assert!(matches!(result_bad_quantity, Err(DataError::Socket(_)))); 
         if let Err(DataError::Socket(s)) = result_bad_quantity {
            assert!(s.contains("Failed to parse quantity"));
        }
    }

     #[test]
    fn test_transform_push_data_v3_api_wrapper_public_aggre_deals() {
        let instrument = TestInstrument { base: "ETH".into(), quote: "USDT".into() };
        let deal_item1 = proto::PublicAggreDealsV3ApiItem {
            price: "3000.1".to_string(),
            quantity: "0.1".to_string(),
            trade_type: 2, // Sell
            time: 1609459300000,
        };
        let deal_item2 = proto::PublicAggreDealsV3ApiItem {
            price: "3000.0".to_string(),
            quantity: "0.05".to_string(),
            trade_type: 1, // Buy
            time: 1609459300100,
        };

        let wrapper = proto::PushDataV3ApiWrapper {
            channel: "spot@public.aggre.deals.v3.api.pb@100ms@ETH_USDT".to_string(),
            symbol: Some("ETH_USDT".to_string()),
            symbol_id: Some("ETH_USDT_ID".to_string()),
            create_time: Some(1609459300200),
            send_time: Some(1609459300250),
            body: Some(proto::push_data_v3_api_wrapper::Body::PublicAggreDeals(
                proto::PublicAggreDealsV3Api {
                    deals: vec![deal_item1.clone(), deal_item2.clone()],
                    event_type: "DEALS".to_string(),
                }
            )),
        };

        let market_iter = MarketIter::<TestInstrument, PublicTrade>::from((ExchangeId::Mexc, instrument.clone(), wrapper));
        let events: Vec<_> = market_iter.0.into_iter().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind.side, Side::Sell);
        assert_eq!(events[1].kind.side, Side::Buy);
    }

    #[test]
    fn test_transform_push_data_v3_api_wrapper_public_deals() {
        let instrument = TestInstrument { base: "LTC".into(), quote: "USDT".into() };
        let deal_item = proto::PublicDealsV3ApiItem {
            price: "150.5".to_string(),
            quantity: "1.2".to_string(),
            trade_type: 1, // Buy
            time: 1609459400000,
        };
         let wrapper = proto::PushDataV3ApiWrapper {
            channel: "spot@public.deals.v3.api.pb@LTC_USDT".to_string(),
            symbol: Some("LTC_USDT".to_string()),
            symbol_id: Some("LTC_USDT_ID".to_string()),
            create_time: Some(1609459400100),
            send_time: Some(1609459400150),
            body: Some(proto::push_data_v3_api_wrapper::Body::PublicDeals(
                proto::PublicDealsV3Api {
                    deals: vec![deal_item.clone()],
                    event_type: "DEALS".to_string(),
                }
            )),
        };

        let market_iter = MarketIter::<TestInstrument, PublicTrade>::from((ExchangeId::Mexc, instrument.clone(), wrapper));
        let events: Vec<_> = market_iter.0.into_iter().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind.price, 150.5);
    }

    #[test]
    fn test_transform_push_data_v3_api_wrapper_no_body() {
        let instrument = TestInstrument { base: "BTC".into(), quote: "USDT".into() };
        let wrapper = proto::PushDataV3ApiWrapper {
            channel: "some_channel".to_string(),
            symbol: Some("BTC_USDT".to_string()),
            symbol_id: Some("BTC_USDT_ID".to_string()),
            create_time: Some(1609459200000),
            send_time: Some(1609459200000),
            body: None,
        };
        let market_iter = MarketIter::<TestInstrument, PublicTrade>::from((ExchangeId::Mexc, instrument, wrapper));
        let events: Vec<_> = market_iter.0.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_transform_push_data_v3_api_wrapper_other_body_type() {
        let instrument = TestInstrument { base: "BTC".into(), quote: "USDT".into() };
        let kline_item = proto::PublicSpotKlineV3Api {
            interval: "Min1".to_string(),
            window_start: 1609459200,
            opening_price: "50000".to_string(),
            closing_price: "50010".to_string(),
            highest_price: "50015".to_string(),
            lowest_price: "49990".to_string(),
            volume: "10".to_string(),
            amount: "500000".to_string(),
            window_end: 1609459260,
        };
        let wrapper = proto::PushDataV3ApiWrapper {
            channel: "spot@public.kline.v3.api@Min1@BTC_USDT".to_string(),
            symbol: Some("BTC_USDT".to_string()),
            symbol_id: Some("BTC_USDT_ID".to_string()),
            create_time: Some(1609459260000),
            send_time: Some(1609459260000),
            body: Some(proto::push_data_v3_api_wrapper::Body::PublicSpotKline(kline_item)),
        };

        let market_iter = MarketIter::<TestInstrument, PublicTrade>::from((ExchangeId::Mexc, instrument, wrapper));
        let events: Vec<_> = market_iter.0.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
        assert!(events.is_empty());
    }
}
