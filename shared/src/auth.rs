use std::borrow::Cow;

use anyhow::{Context as _, bail};
use reqwest::Url;

use super::AuthMethod;
use crate::utils::time::timestamp_now;

// TODO: optimize by making query sorted in the first place, in send_request?
// But other auth methods maybe don't want this
// Also will cause request query to be sorted
fn timestamped_sorted_params(url: &Url, timestamp: &str) -> String {
    let mut query: Vec<_> = url.query_pairs().collect();
    query.push((Cow::Borrowed("signTimestamp"), Cow::Borrowed(timestamp)));
    query.sort_unstable();
    let mut ser = form_urlencoded::Serializer::for_suffix(String::new(), 0);
    ser.extend_pairs(query);
    ser.finish()
}

mod hmac_sha256 {
    use base64::prelude::{BASE64_STANDARD, Engine as _};
    use hmac::digest::Mac as _;

    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    // TODO: reuse buffers?
    pub(super) fn sign_payload(payload: &str, secret_key: &str) -> String {
        // call hash function
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let mac_output = mac.finalize();

        // encode hash with base64
        BASE64_STANDARD.encode(mac_output.into_bytes())
    }
}

impl AuthMethod {
    pub(super) fn apply(&self, req: &mut reqwest::Request) -> anyhow::Result<()> {
        match self {
            AuthMethod::HmacSha256 {
                api_key,
                secret_key,
            } => {
                let method = req.method().as_str();

                // TODO: implement POST, PUT, DELETE
                if method != "GET" {
                    bail!("only GET method implemented for HmacSha256 authentication mode yet");
                }

                // url without authority, domain and query
                let path = req.url().path();

                let timestamp = timestamp_now().to_string();

                let params = timestamped_sorted_params(req.url(), &timestamp);

                let payload = format!("{method}\n{path}\n{params}");

                let sign = hmac_sha256::sign_payload(&payload, secret_key);

                let headers = req.headers_mut();
                headers.append(
                    "key",
                    api_key.try_into().context("header value from public key")?,
                );
                headers.append(
                    "signature",
                    sign.try_into().context("header value from signature")?,
                );
                headers.append(
                    "signTimestamp",
                    timestamp
                        .try_into()
                        .context("header value from timestamp")?,
                );
            }
        }
        Ok(())
    }
}
