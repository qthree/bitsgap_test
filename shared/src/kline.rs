/// Структура KL как в ТЗ тестового задания
#[derive(Debug, serde::Serialize)]
pub struct Kline {
    /// название пары как у нас
    pub pair: String,
    /// период формирования свечи (1m, 15m, 1h, 1d)
    pub time_frame: String,
    /// open - цена открытия
    pub o: f64,
    /// high - максимальная цена
    pub h: f64,
    /// low - минимальная цена
    pub l: f64,
    /// close - цена закрытия
    pub c: f64,
    /// время unix начала формирования свечки
    pub utc_begin: i64,
    pub volume_bs: VBS,
}

#[derive(Debug, serde::Serialize)]
pub struct VBS {
    /// объём покупок в базовой валюте
    pub buy_base: f64,
    /// объём продаж в базовой валюте
    pub sell_base: f64,
    /// объём покупок в котируемой валюте
    pub buy_quote: f64,
    /// объём продаж в котируемой валюте
    pub sell_quote: f64,
}
