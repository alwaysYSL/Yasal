use crate::core::frecency::FrecencyStore;
use crate::core::plugin::{Plugin, QueryResult};
use std::sync::Arc;

pub struct Dispatcher {
    plugins: Vec<Arc<dyn Plugin>>,
    frecency: FrecencyStore,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            frecency: FrecencyStore::new(),
        }
    }

    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn remove(&mut self, plugin_id: &str) {
        self.plugins.retain(|plugin| plugin.id() != plugin_id);
    }

    /// Returns the symbolic icon or mode name for the active query prefix.
    pub fn detect_mode_icon(&self, query: &str) -> &'static str {
        let trimmed = query.trim_start();
        if trimmed.starts_with('=') {
            return "calc";
        } else if trimmed.starts_with('>') {
            return "shell";
        } else if trimmed.starts_with("??") {
            return "web";
        } else if trimmed.starts_with("file ") || trimmed.eq_ignore_ascii_case("file") {
            return "file";
        } else if trimmed.starts_with("cb ") || trimmed.eq_ignore_ascii_case("cb") {
            return "clipboard";
        } else if trimmed.eq_ignore_ascii_case("ip") || trimmed.eq_ignore_ascii_case("network") {
            return "network";
        } else if trimmed.eq_ignore_ascii_case("w") || trimmed.starts_with("w ") {
            return "window";
        } else if trimmed.starts_with('$') || trimmed.eq_ignore_ascii_case("settings") {
            return "settings";
        }
        "search"
    }

    /// Dispatches query and returns sorted, scored results (up to 8 items).
    pub fn dispatch(&self, query: &str) -> Vec<QueryResult> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            // Return top frequent/recent items if available
            return self.get_empty_state_results();
        }

        // 1. Prefix-based routing
        for plugin in &self.plugins {
            if let Some(prefix) = plugin.prefix() {
                let textual_prefix = prefix.chars().all(char::is_alphanumeric);
                let remainder = trimmed.strip_prefix(prefix);
                let matches = remainder.is_some_and(|rest| {
                    !textual_prefix || rest.is_empty() || rest.starts_with(char::is_whitespace)
                });
                if matches {
                    return plugin.query(remainder.unwrap_or_default().trim());
                }
            }
        }

        // 2. Global broadcast to all global plugins
        let mut results = Vec::new();
        for plugin in &self.plugins {
            if plugin.is_global() {
                let mut plugin_results = plugin.query(trimmed);
                // Apply frecency multipliers to scores
                for res in &mut plugin_results {
                    let mult = self.frecency.get_frecency_multiplier(&res.id);
                    res.score = ((res.score as f64) * mult) as u32;
                }
                results.extend(plugin_results);
            }
        }

        // A file lookup is useful only after the ordinary global plugins have
        // no local match. This keeps filesystem indexing lazy during startup
        // and while common app or command searches are being typed.
        if trimmed.chars().count() >= 3
            && results.iter().all(|result| result.category == "Web")
            && !trimmed.contains("://")
        {
            if let Some(file_search) = self
                .plugins
                .iter()
                .find(|plugin| plugin.id() == "file_search")
            {
                results.extend(file_search.query(trimmed));
            }
        }

        // Sort descending by score
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(8);
        results
    }

    /// Records when the user executes an item so it ranks higher in future searches.
    pub fn record_use(&mut self, item_id: &str) {
        self.frecency.record_use(item_id);
    }

    fn get_empty_state_results(&self) -> Vec<QueryResult> {
        let top_ids = self.frecency.get_top_frequent(5);
        if top_ids.is_empty() {
            // When frecency history is empty on first run, show the first 5 indexed apps
            for plugin in &self.plugins {
                if plugin.id() == "app_launcher" {
                    let mut apps = plugin.query("");
                    apps.truncate(5);
                    return apps;
                }
            }
            return Vec::new();
        }

        let mut results = Vec::new();
        for plugin in &self.plugins {
            if plugin.is_global() {
                let all_items = plugin.query("");
                for item in all_items {
                    if top_ids.contains(&item.id) {
                        results.push(item);
                    }
                }
            }
        }
        results.truncate(5);
        results
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}
