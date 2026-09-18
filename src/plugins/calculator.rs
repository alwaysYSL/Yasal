use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::plugins::app_launcher::set_clipboard_text;
use std::sync::Arc;

pub struct CalculatorPlugin;

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self
    }

    fn looks_like_math(input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
        let has_math_op = trimmed.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '%'));
        has_digit && has_math_op
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CalculatorPlugin {
    fn id(&self) -> &str {
        "calculator"
    }

    fn name(&self) -> &str {
        "Calculator"
    }

    fn prefix(&self) -> Option<&str> {
        Some("=")
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let clean_expr = input.trim_start_matches('=').trim();
        if clean_expr.is_empty() {
            return Vec::new();
        }

        // If not explicitly prefixed, only evaluate if it looks like math
        if !input.starts_with('=') && !Self::looks_like_math(clean_expr) {
            return Vec::new();
        }

        if let Ok(value) = evalexpr::eval(clean_expr) {
            let result_str = value.to_string();
            let copy_str = result_str.clone();

            let copy_action: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                let _ = set_clipboard_text(&copy_str);
            });

            return vec![QueryResult {
                id: format!("calc:{}", result_str),
                title: format!("= {}", result_str),
                subtitle: Some(format!("Ekspresi: {} (Tekan Enter untuk salin)", clean_expr)),
                category: "Kalkulator".to_string(),
                icon: ItemIcon::Symbolic("calc"),
                score: 950, // High priority when math is valid
                primary_action: copy_action.clone(),
                secondary_action: Some(ItemAction {
                    label: "Salin Hasil".to_string(),
                    icon: "📋",
                    shortcut_hint: Some("Enter / Ctrl+Enter".to_string()),
                    action: copy_action,
                }),
                additional_actions: Vec::new(),
            }];
        }

        Vec::new()
    }
}
