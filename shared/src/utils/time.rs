use std::{fmt, str::FromStr, time::Duration};

use anyhow::Context;
use jiff::{Span, Timestamp};

/// return UNIX timestamp in milliseconds since 'Thu Jan 01 1970 00:00:00.000'
/// u64 is enough for another ~585 million years
fn millis_since_unix_epoch(ts: Timestamp) -> u64 {
    let duration = ts.duration_since(Timestamp::UNIX_EPOCH);
    duration.as_millis().clamp(0, u64::MAX as _) as _
}

pub fn timestamp_now() -> u64 {
    millis_since_unix_epoch(Timestamp::now())
}

pub fn timestamp_parse(ts: &str) -> anyhow::Result<u64> {
    let ts: Timestamp = ts.parse().context("parse timestamp")?;
    Ok(millis_since_unix_epoch(ts))
}

pub fn timestamp_display(ts: u64) -> impl fmt::Display {
    Timestamp::UNIX_EPOCH.saturating_add(Duration::from_millis(ts))
}

// Converts "2025-02-06T22:05:59.999Z" to "2025-02-06T22:06:00.000Z"
// Because polonies likes round start_time
// {
//    "code" : 24105,
//    "message" : "Invalid start time!"
// }
// Alternatively, we can pass our Interval, to round up more smartly, not just to next second
pub fn timestamp_ceil(ts: u64) -> u64 {
    (ts + 1) / 1000 * 1000
}

/// `std::time::Duration` with `impl FromStr` of `jiff::Span`
#[derive(Debug, Clone, Copy)]
pub struct SpanDuration(pub Duration);

impl fmt::Display for SpanDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(span) = Span::try_from(self.0) else {
            return Err(fmt::Error);
        };
        span.fmt(f)
    }
}

impl From<Duration> for SpanDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<SpanDuration> for Duration {
    fn from(value: SpanDuration) -> Self {
        value.0
    }
}

impl FromStr for SpanDuration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let span: Span = s.parse().context("parse string as span")?;
        span.try_into()
            .map(Self)
            .context("convert span to duration")
    }
}
