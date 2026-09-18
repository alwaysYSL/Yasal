use crate::core::plugin::{ItemIcon, Plugin, QueryResult};
use std::sync::Arc;

pub struct WebSearchPlugin {
    pub search_engine: String,
}

impl WebSearchPlugin {
    pub fn new(search_engine: &str) -> Self {
        Self {
            search_engine: search_engine.to_string(),
        }
    }

    fn is_url(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("www.")
            || lower.starts_with("mailto:")
            || (lower.contains('.') && !lower.contains(' ') && (lower.ends_with(".com") || lower.ends_with(".org") || lower.ends_with(".net") || lower.ends_with(".io") || lower.ends_with(".dev") || lower.ends_with(".id")))
    }

    fn build_search_url(&self, query: &str) -> String {
        let encoded = urlencoding::encode(query);
        match self.search_engine.to_lowercase().as_str() {
            "duckduckgo" => format!("https://duckduckgo.com/?q={}", encoded),
            "bing" => format!("https://www.bing.com/search?q={}", encoded),
            _ => format!("https://www.google.com/search?q={}", encoded),
        }
    }
}

// Simple fallback urlencoder
mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut result = String::new();
        for byte in data.bytes() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => result.push('+'),
                _ => result.push_str(&format!("%{:02X}", byte)),
            }
        }
        result
    }
}

impl Default for WebSearchPlugin {
    fn default() -> Self {
        Self::new("Google")
    }
}

impl Plugin for WebSearchPlugin {
    fn id(&self) -> &str {
        "web_search"
    }

    fn name(&self) -> &str {
        "Web Search"
    }

    fn prefix(&self) -> Option<&str> {
        Some("??")
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let is_prefix = input.starts_with("??");
        let query_text = if is_prefix {
            trimmed.trim_start_matches('?').trim()
        } else {
            trimmed
        };

        if query_text.is_empty() {
            return Vec::new();
        }

        if Self::is_url(query_text) {
            let url = if query_text.starts_with("http://") || query_text.starts_with("https://") || query_text.starts_with("mailto:") {
                query_text.to_string()
            } else {
                format!("https://{}", query_text)
            };
            let url_clone = url.clone();

            return vec![QueryResult {
                id: format!("url:{}", url),
                title: format!("Buka URL {}", url),
                subtitle: Some("Buka tautan web di browser default".to_string()),
                category: "Web".to_string(),
                icon: ItemIcon::Symbolic("web"),
                score: 800,
                primary_action: Arc::new(move || {
                    let _ = open::that_detached(&url_clone);
                }),
                secondary_action: None,
                additional_actions: Vec::new(),
            }];
        }

        let search_url = self.build_search_url(query_text);
        let url_clone = search_url.clone();
        let engine_name = self.search_engine.clone();

        vec![QueryResult {
            id: format!("web:{}", query_text),
            title: format!("Cari \"{}\" di {}", query_text, engine_name),
            subtitle: Some("Buka pencarian web di browser default".to_string()),
            category: "Web".to_string(),
            icon: ItemIcon::Symbolic("web"),
            score: if is_prefix { 900 } else { 10 }, // Low global fallback, high when prefixed
            primary_action: Arc::new(move || {
                let _ = open::that_detached(&url_clone);
            }),
            secondary_action: None,
            additional_actions: Vec::new(),
        }]
    }
}
