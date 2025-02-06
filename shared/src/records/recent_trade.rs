/// Структура RT как в ТЗ тестового задания
#[derive(Debug, serde::Serialize)]
pub struct RecentTrade {
    /// id транзакции
    pub tid: String,
    /// название валютной пары (как у нас)
    pub pair: String,
    /// цена транзакции
    pub price: String,
    /// объём транзакции в базовой валюте
    pub amount: String,
    /// как биржа засчитала эту сделку (как buy или как sell)
    pub side: String,
    /// время UTC UnixNano
    pub timestamp: i64,
}
