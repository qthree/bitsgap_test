use bitsgap_shared::{
    interval::{IntervalKind, IntervalsDict},
    utils::ValueLabel,
};

pub struct WsCandlesChannels;
impl ValueLabel for WsCandlesChannels {
    type Value = IntervalsDict;
}

// TOOD: load from config
pub fn all_ws_candles_channels() -> anyhow::Result<IntervalsDict> {
    IntervalsDict::default()
        .with(
            IntervalKind::Minute,
            [
                (1, "candles_minute_1"),
                (5, "candles_minute_5"),
                (10, "candles_minute_10"),
                (15, "candles_minute_15"),
                (30, "candles_minute_30"),
            ],
        )?
        .with(
            IntervalKind::Hour,
            [
                (1, "candles_hour_1"),
                (2, "candles_hour_2"),
                (4, "candles_hour_4"),
                (6, "candles_hour_6"),
                (12, "candles_hour_12"),
            ],
        )?
        .with(
            IntervalKind::Day,
            [(1, "candles_day_1"), (3, "candles_day_3")],
        )?
        .with(IntervalKind::Week, [(1, "candles_week_1")])?
        .with(IntervalKind::Month, [(1, "candles_month_1")])
}

pub fn supported_ws_candles_channels(
    supported_intervals: &IntervalsDict,
) -> anyhow::Result<IntervalsDict> {
    let all = all_ws_candles_channels()?;
    let mut res = IntervalsDict::default();
    for (interval, alias) in all.iter() {
        if supported_intervals.to_alias(interval).is_some() {
            res.add(interval, alias.to_string())?;
        }
    }
    Ok(res)
}
