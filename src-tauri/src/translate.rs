use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

const GOOGLE_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Reused so a debounce tick does not redo the TLS handshake.
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
}

/// Dictionary entry for a single word: "noun" -> ["run", "jog"].
#[derive(Serialize, Debug, PartialEq)]
pub struct Meaning {
    pub pos: String,
    pub terms: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Translation {
    pub text: String,
    pub detected: String,
    /// Other renderings of the same text (only when it is a single segment).
    pub alternatives: Vec<String>,
    pub meanings: Vec<Meaning>,
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
            ("dt", "t"),  // translation
            ("dt", "at"), // alternatives
            ("dt", "bd"), // dictionary
            ("q", text),
        ])
        .send()
        .await
        // reqwest's Display appends the url, which carries the user's text
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

fn status_error_message(status: reqwest::StatusCode) -> String {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "translation rate limit reached, try again shortly".to_string()
    } else {
        format!("translation failed (status {})", status.as_u16())
    }
}

/// `[[["hello","merhaba",..],["world","dünya",..]], dictionary, "tr", .., .., alternatives, ..]`
/// where dictionary is `[["noun", ["merhaba", ..], ..], ..]` and alternatives is
/// `[[source, _, [["merhaba", ..], ["selam", ..]], ..]]`, one entry per segment.
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

    // Both extras are optional garnish: a shape we don't recognise just means none.
    let strings = |v: Option<&serde_json::Value>| -> Vec<String> {
        v.and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };

    let meanings = v
        .get(1)
        .and_then(|d| d.as_array())
        .map(|entries| {
            entries
                .iter()
                .map(|e| Meaning {
                    pos: e.get(0).and_then(|p| p.as_str()).unwrap_or_default().to_string(),
                    terms: strings(e.get(1)),
                })
                .filter(|m| !m.terms.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let segments = v.get(5).and_then(|a| a.as_array());
    let alternatives = match segments {
        Some(s) if s.len() == 1 => s[0]
            .get(2)
            .and_then(|a| a.as_array())
            .map(|alts| {
                let mut seen = vec![text.trim().to_string()];
                for a in alts {
                    if let Some(t) = a.get(0).and_then(|t| t.as_str()) {
                        if !seen.iter().any(|x| x == t.trim()) {
                            seen.push(t.trim().to_string());
                        }
                    }
                }
                seen.split_off(1)
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    Ok(Translation { text, detected, alternatives, meanings })
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
    fn reads_alternatives_and_meanings() {
        let body = r#"[[["koşmak","run",null,null,10]],[["verb",["koşmak","çalıştırmak"],[["koşmak",["run"],null,0.5]],"run",1]],"en",null,null,[["run",null,[["koşmak",1000,true,false],["çalıştırmak",900,true,false],["koşu",800,true,false]],[[0,3]],"run",0,0]]]"#;
        let t = parse_google(body).unwrap();
        assert_eq!(t.text, "koşmak");
        assert_eq!(t.alternatives, vec!["çalıştırmak", "koşu"]); // main one dropped
        assert_eq!(t.meanings, vec![Meaning { pos: "verb".into(), terms: vec!["koşmak".into(), "çalıştırmak".into()] }]);
    }

    #[test]
    fn multi_segment_text_has_no_alternatives() {
        let body = r#"[[["A. ","a",null,null,1],["B.","b",null,null,1]],null,"en",null,null,[["A.",null,[["x",1]]],["B.",null,[["y",1]]]]]"#;
        let t = parse_google(body).unwrap();
        assert!(t.alternatives.is_empty() && t.meanings.is_empty());
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
