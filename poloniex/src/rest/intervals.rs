use bitsgap_shared::interval::{IntervalKind, IntervalsDict};

// TOOD: load from config
pub fn exchange_intervals() -> anyhow::Result<IntervalsDict> {
    IntervalsDict::default()
        .with(
            IntervalKind::Minute,
            [
                (1, "MINUTE_1"),
                (5, "MINUTE_5"),
                (10, "MINUTE_10"),
                (15, "MINUTE_15"),
                (30, "MINUTE_30"),
            ],
        )?
        .with(
            IntervalKind::Hour,
            [
                (1, "HOUR_1"),
                (2, "HOUR_2"),
                (4, "HOUR_4"),
                (6, "HOUR_6"),
                (12, "HOUR_12"),
            ],
        )?
        .with(IntervalKind::Day, [(1, "DAY_1"), (3, "DAY_3")])?
        .with(IntervalKind::Week, [(1, "WEEK_1")])?
        .with(IntervalKind::Month, [(1, "MONTH_1")])
}
