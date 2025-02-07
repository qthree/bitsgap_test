use bitsgap_shared::interval::Interval;

pub enum Channel {
    Candles(Interval),
    Trades,
}
