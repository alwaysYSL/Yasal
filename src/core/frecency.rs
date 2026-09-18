use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const HALF_LIFE_HOURS: f64 = 72.0; // 3 days half-life decay
const LAMBDA: f64 = 0.693147 / (HALF_LIFE_HOURS * 3600.0); // decay constant per second

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct FrecencyData {
    pub history: HashMap<String, Vec<u64>>, // item id -> unix timestamps in seconds
}

pub struct FrecencyStore {
    data: FrecencyData,
    file_path: Option<PathBuf>,
}

impl FrecencyStore {
    pub fn new() -> Self {
        let file_path = dirs::config_dir().map(|d| d.join("Yasal").join("frecency.json"));
        let data = if let Some(ref path) = file_path {
            if let Ok(content) = fs::read_to_string(path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                FrecencyData::default()
            }
        } else {
            FrecencyData::default()
        };

        Self { data, file_path }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Records usage of an item by its unique ID.
    pub fn record_use(&mut self, item_id: &str) {
        let now = Self::now_secs();
        let timestamps = self.data.history.entry(item_id.to_string()).or_default();
        timestamps.push(now);

        // Keep last 30 usages per item to prevent unbounded growth
        if timestamps.len() > 30 {
            timestamps.remove(0);
        }

        self.save();
    }

    /// Calculates the current frecency weight for an item.
    pub fn get_frecency_multiplier(&self, item_id: &str) -> f64 {
        let now = Self::now_secs();
        if let Some(timestamps) = self.data.history.get(item_id) {
            let mut score = 0.0;
            for &ts in timestamps {
                let delta_t = if now >= ts { (now - ts) as f64 } else { 0.0 };
                score += (-LAMBDA * delta_t).exp();
            }
            1.0 + (score * 0.25) // Boost factor (1.0 = base, up to 5.0x for frequent items)
        } else {
            1.0
        }
    }

    /// Returns the top N most frequent/recent item IDs (for empty state).
    pub fn get_top_frequent(&self, limit: usize) -> Vec<String> {
        let mut scored: Vec<(String, f64)> = self
            .data
            .history
            .keys()
            .map(|id| (id.clone(), self.get_frecency_multiplier(id)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(id, _)| id).collect()
    }

    fn save(&self) {
        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string(&self.data) {
                let _ = fs::write(path, json);
            }
        }
    }
}

impl Default for FrecencyStore {
    fn default() -> Self {
        Self::new()
    }
}
