//! Translation backend. Providers are a closed set: adding one means adding a
//! `Provider` variant, a `match` arm in `fetch`, and its own request function.

use serde::{Deserialize, Serialize};

const GOOGLE_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";

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
    let body = reqwest::Client::new()
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
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    parse_google(&body)
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
}
