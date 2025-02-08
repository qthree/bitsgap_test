pub mod context;
pub mod rest;
pub mod units;
pub mod ws;

pub const TEST_TASK_SYMBOLS: &[&str] =
    &["BTC_USDT", "TRX_USDT", "ETH_USDT", "DOGE_USDT", "BCH_USDT"];

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use bitsgap_shared::{ApiConfig, ApiFactory, ApiRequester, AuthMethod};
    use context::PoloniexContext;

    use super::*;

    static LOGGER: Once = Once::new();

    pub(crate) fn init_logger() {
        LOGGER.call_once(|| {
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .init();
        });
    }

    pub(crate) fn poloniex_requester() -> ApiRequester<PoloniexContext> {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let context = PoloniexContext::init(false).unwrap();
        ApiFactory::init(Default::default())
            .unwrap()
            .make_requester(
                ApiConfig {
                    base_url: "https://api.poloniex.com".try_into().unwrap(),
                    auth: AuthMethod::HmacSha256 {
                        api_key,
                        secret_key,
                    },
                },
                context,
            )
    }
}
