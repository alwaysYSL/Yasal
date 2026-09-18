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

    /// Returns the symbolic icon or mode name for the active query prefix.
    pub fn detect_mode_icon(&self, query: &str) -> &'static str {
        let trimmed = query.trim_start();
        if trimmed.starts_with('=') {
            return "calc";
        } else if trimmed.starts_with('>') {
            return "shell";
        } else if trimmed.starts_with("??") {
            return "web";
        } else if trimmed.eq_ignore_ascii_case("settings") {
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
                if trimmed.starts_with(prefix) {
                    let sub_query = trimmed[prefix.len()..].trim();
                    return plugin.query(sub_query);
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
