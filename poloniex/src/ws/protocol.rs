use serde_json::Value;

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ClientMsg {
    Ping,
    Subscribe {
        channel: Vec<String>,
        symbols: Vec<String>,
    },
    Unsubscribe {
        channel: Vec<String>,
        symbols: Vec<String>,
    },
    UnsubscribeAll,
    ListSubscriptions,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServerMsg {
    Stream(ServerStream),
    Event(ServerEvent),
    Subscriptions { subscriptions: Vec<String> },
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct ServerStream {
    data: Vec<Value>,
    channel: String,
}
impl ServerStream {
    pub fn into_msg(self) -> ServerMsg {
        ServerMsg::Stream(self)
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ServerEvent {
    Pong,
    Subscribe {
        channel: String,
    },
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe {
        channel: String,
    },
    #[serde(rename = "UNSUBSCRIBE_ALL")]
    UnsubscribeAll {
        channel: String,
    },
    Error {
        message: ServerError,
    },
}
impl ServerEvent {
    pub fn into_msg(self) -> ServerMsg {
        ServerMsg::Event(self)
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServerError {
    Kind(ServerErrorKind),
    Unknown(String),
}
impl ServerError {
    pub fn into_msg(self) -> ServerMsg {
        ServerEvent::Error { message: self }.into_msg()
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub enum ServerErrorKind {
    #[serde(rename = "Subscription failed")]
    SubscriptionFailed,
    #[serde(rename = "Already subscribed")]
    AlreadySubscribed,
    #[serde(rename = "Not subscribed")]
    NotSubscribed,
    #[serde(rename = "Bad request")]
    BadRequest,
    #[serde(rename = "Request failed")]
    RequestFailed,
}
impl ServerErrorKind {
    pub fn into_error(self) -> ServerError {
        ServerError::Kind(self)
    }

    pub fn into_msg(self) -> ServerMsg {
        self.into_error().into_msg()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use anyhow::Context;

    use super::*;
    use crate::ws::candles::CandlesMessage;

    #[track_caller]
    fn assert_eq<T: serde::de::DeserializeOwned + PartialEq + fmt::Debug>(msg: T, json: &str) {
        let parsed: T = serde_json::from_str(json)
            .with_context(|| json.to_string())
            .unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_heartbeats() {
        assert_eq(ClientMsg::Ping, r#"{"event": "ping"}"#);
        assert_eq(ServerEvent::Pong.into_msg(), r#"{"event": "pong"}"#);
    }

    #[test]
    fn test_subscribe() {
        assert_eq(
            ClientMsg::Subscribe {
                channel: vec!["<channel>".into()],
                symbols: vec!["<symbol1>".into(), "<symbol2>".into(), "<symbol3>".into()],
            },
            r#"
            {
                "event": "subscribe",
                "channel": ["<channel>"],
                "symbols": [
                    "<symbol1>",
                    "<symbol2>",
                    "<symbol3>"
                ]
            }
            "#,
        );
        assert_eq(
            ClientMsg::Subscribe {
                channel: vec!["<channel>".into()],
                symbols: vec!["all".into()],
            },
            r#"
            {
                "event": "subscribe",
                "channel": ["<channel>"],
                "symbols": ["all"]
            }
            "#,
        );

        assert_eq(
            ServerEvent::Subscribe {
                channel: "<channel>".into(),
            }
            .into_msg(),
            r#"
            {
                "event": "subscribe",
                "channel": "<channel>"
            }
            "#,
        );
    }

    #[test]
    fn test_unsubscribe() {
        assert_eq(
            ClientMsg::Unsubscribe {
                channel: vec!["<channel>".into()],
                symbols: vec!["<symbol>".into()],
            },
            r#"
            {
                "event": "unsubscribe",
                "channel": ["<channel>"],
                "symbols": [
                "<symbol>"
                ]
            }
            "#,
        );
        assert_eq(
            ClientMsg::Unsubscribe {
                channel: vec!["<channel>".into()],
                symbols: vec!["all".into()],
            },
            r#"
            {
                "event": "unsubscribe",
                "channel": ["<channel>"],
                "symbols": ["all"]
            }
            "#,
        );

        assert_eq(
            ServerEvent::Unsubscribe {
                channel: "<channel>".into(),
            }
            .into_msg(),
            r#"
            {
                "channel": "<channel>",
                "event": "UNSUBSCRIBE"
            }          
            "#,
        );
    }

    #[test]
    fn test_unsubscribe_all() {
        assert_eq(
            ClientMsg::UnsubscribeAll,
            r#"
            {
                "event": "unsubscribe_all"
            }
            "#,
        );

        assert_eq(
            ServerEvent::UnsubscribeAll {
                channel: "ALL".into(),
            }
            .into_msg(),
            r#"
            {
                "channel": "ALL",
                "event": "UNSUBSCRIBE_ALL"
            }
            "#,
        );
    }

    #[test]
    fn test_list_subscriptions() {
        assert_eq(
            ClientMsg::ListSubscriptions,
            r#"
            {
                "event": "list_subscriptions"
            }
            "#,
        );

        assert_eq(
            ServerMsg::Subscriptions {
                subscriptions: vec!["<channel>".into()],
            },
            r#"
            {
                "subscriptions": ["<channel>"]
            }
            "#,
        );
    }

    #[test]
    fn test_errors() {
        assert_eq(
            ServerError::Unknown("Error Message".into()).into_msg(),
            r#"
            {
                "event": "error",
                "message": "Error Message"
            }
            "#,
        );
        assert_eq(
            ServerErrorKind::SubscriptionFailed.into_msg(),
            r#"
            {
                "event": "error",
                "message": "Subscription failed"
            }
            "#,
        );
        assert_eq(
            ServerErrorKind::AlreadySubscribed.into_msg(),
            r#"
            {
                "event": "error",
                "message": "Already subscribed"
            }
            "#,
        );
        assert_eq(
            ServerErrorKind::BadRequest.into_msg(),
            r#"
            {
                "event": "error",
                "message": "Bad request"
            }
            "#,
        );
        assert_eq(
            ServerErrorKind::RequestFailed.into_msg(),
            r#"
            {
                "event": "error",
                "message": "Request failed"
            }
            "#,
        );
        assert_eq(
            ServerErrorKind::NotSubscribed.into_msg(),
            r#"
            {
                "event": "error",
                "message": "Not subscribed"
            }
            "#,
        );
    }

    #[test]
    fn test_stream() {
        assert_eq(
            ServerStream {
                channel: "candles_minute_1".into(),
                data: vec![
                    serde_json::to_value(&CandlesMessage {
                        symbol: "BTC_USDT".into(),
                        amount: "0".into(),
                        high: "9999.07".into(),
                        quantity: "0".into(),
                        trade_count: 0,
                        low: "9999.07".into(),
                        close_time: 1648057199999,
                        start_time: 1648057140000,
                        close: "9999.07".into(),
                        open: "9999.07".into(),
                        record_time: 1648057141081,
                    })
                    .unwrap(),
                ],
            }
            .into_msg(),
            r#"
            {
                "channel": "candles_minute_1",
                "data": [{
                    "symbol": "BTC_USDT",
                    "amount": "0",
                    "high": "9999.07",
                    "quantity": "0",
                    "tradeCount": 0,
                    "low": "9999.07",
                    "closeTime": 1648057199999,
                    "startTime": 1648057140000,
                    "close": "9999.07",
                    "open": "9999.07",
                    "ts": 1648057141081
                }]
            }
        "#,
        );
    }
}
