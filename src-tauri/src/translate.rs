//! Translation backend. Providers are a closed set: adding one means adding a
//! `Provider` variant, a `match` arm in `fetch`, and its own request function.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

const GOOGLE_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

// Built once and reused: a fresh `reqwest::Client` per call would open a new
// connection pool and redo the TLS handshake on every debounce tick.
fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build reqwest client")
    })
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Google,
    // Yandex,  // new engine: add the variant, a `match` arm below, and a fetch fn
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Translation {
    pub text: String,
    pub detected: String,
}

pub async fn fetch(
    provider: Provider,
    text: &str,
    from: &str,
    to: &str,
) -> Result<Translation, String> {
    match provider {
        Provider::Google => google(text, from, to).await,
    }
}

async fn google(text: &str, from: &str, to: &str) -> Result<Translation, String> {
    let resp = client()
        .get(GOOGLE_ENDPOINT)
        .query(&[
            ("client", "gtx"),
            ("sl", from),
            ("tl", to),
            ("dt", "t"),
            ("q", text),
        ])
        .send()
        .await
        // reqwest's Display appends " for url (...)", and the url contains the
        // user's typed text as a query param — never surface it to the UI.
        .map_err(|_| "translation request failed".to_string())?;

    let status = resp.status();
    if !status.is_success() {
        return Err(status_error_message(status));
    }

    let body = resp
        .text()
        .await
        .map_err(|_| "translation request failed".to_string())?;
    parse_google(&body)
}

// Pure so it's testable without a network call: maps a failing status to a
// short, human-readable message that never contains request/response data.
fn status_error_message(status: reqwest::StatusCode) -> String {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "translation rate limit reached, try again shortly".to_string()
    } else {
        format!("translation failed (status {})", status.as_u16())
    }
}

// The endpoint answers with a bare nested array:
//   [[["hello","merhaba",..],["world","dünya",..]], null, "tr", ...]
// -> translated text = concat of [0][i][0], detected language = [2]
pub fn parse_google(body: &str) -> Result<Translation, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("bad translate response: {e}"))?;

    let chunks = v
        .get(0)
        .and_then(|c| c.as_array())
        .ok_or("bad translate response: no chunks")?;

    let mut text = String::new();
    for c in chunks {
        let part = c
            .get(0)
            .and_then(|s| s.as_str())
            .ok_or("bad translate response: chunk is not text")?;
        text.push_str(part);
    }
    if text.is_empty() {
        return Err("bad translate response: empty translation".into());
    }

    let detected = v
        .get(2)
        .and_then(|s| s.as_str())
        .unwrap_or("auto")
        .to_string();

    Ok(Translation { text, detected })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_chunk() {
        let body = r#"[[["hello world","merhaba dünya",null,null,10]],null,"tr",null,null,null,null,[]]"#;
        let t = parse_google(body).unwrap();
        assert_eq!(t.text, "hello world");
        assert_eq!(t.detected, "tr");
    }

    #[test]
    fn joins_multiple_chunks_in_order() {
        let body = r#"[[["Hello. ","Merhaba. ",null,null,3],["How are you?","Nasılsın?",null,null,3]],null,"tr",null,null,null,null,[]]"#;
        let t = parse_google(body).unwrap();
        assert_eq!(t.text, "Hello. How are you?");
    }

    #[test]
    fn rejects_unexpected_shape() {
        assert!(parse_google(r#"{"error":"quota"}"#).is_err());
        assert!(parse_google("not json at all").is_err());
        assert!(parse_google("[]").is_err());
    }

    #[test]
    fn status_error_message_never_leaks_query_text() {
        let query_text = "my bank password is hunter2";
        let msg = status_error_message(reqwest::StatusCode::TOO_MANY_REQUESTS);
        assert!(!msg.contains(query_text));
        assert!(!msg.contains("url"));
        assert!(msg.to_lowercase().contains("rate limit"));

        let msg = status_error_message(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!msg.contains(query_text));
        assert!(msg.contains("500"));
    }
}
