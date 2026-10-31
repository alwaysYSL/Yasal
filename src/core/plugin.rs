use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemIcon {
    App(String),            // Path to icon png or exe path
    Symbolic(&'static str), // "calc", "web", "shell", "settings", "trash", "power", "lock", "moon"
    File(String),           // Path to file
}

#[derive(Clone)]
pub struct ItemAction {
    pub label: String,
    pub icon: &'static str,
    pub shortcut_hint: Option<String>,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

#[derive(Clone)]
pub struct QueryResult {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: String,
    pub icon: ItemIcon,
    pub score: u32,
    pub primary_action: Arc<dyn Fn() + Send + Sync>,
    pub secondary_action: Option<ItemAction>,
    pub additional_actions: Vec<ItemAction>,
}

pub trait Plugin: Send + Sync {
    /// Unique identifier for this plugin (e.g. "app_launcher", "calc", "web")
    fn id(&self) -> &str;

    /// Human-friendly display name
    fn name(&self) -> &str;

    /// Optional prefix trigger (e.g. "=" for calc, ">" for shell, "??" for web)
    fn prefix(&self) -> Option<&str> {
        None
    }

    /// Whether this plugin responds to global (unprefixed) searches
    fn is_global(&self) -> bool {
        false
    }

    /// Query the plugin with input and return scored results
    fn query(&self, input: &str) -> Vec<QueryResult>;

    /// Called instead of `query` when the user typed this plugin's prefix, with the
    /// prefix already removed. Plugins whose unprefixed `query` is deliberately strict
    /// (to avoid noise in global search) can override this to be more permissive.
    fn query_prefixed(&self, input: &str) -> Vec<QueryResult> {
        self.query(input)
    }

    /// Looks up one of this plugin's results by id, e.g. to rebuild the items the user
    /// opens most. The default scans `query("")`; plugins whose empty query is capped
    /// should override it.
    fn find_by_id(&self, id: &str) -> Option<QueryResult> {
        self.query("").into_iter().find(|result| result.id == id)
    }
}
