use crate::core::plugin::{ItemIcon, Plugin, QueryResult};
use std::process::Command;
use std::sync::Arc;

pub struct ShellCommandPlugin;

impl ShellCommandPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShellCommandPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ShellCommandPlugin {
    fn id(&self) -> &str {
        "shell_command"
    }

    fn name(&self) -> &str {
        "Shell Command"
    }

    fn prefix(&self) -> Option<&str> {
        Some(">")
    }

    fn is_global(&self) -> bool {
        false
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let cmd = input.trim_start_matches('>').trim();
        if cmd.is_empty() {
            return Vec::new();
        }

        let cmd_clone = cmd.to_string();

        vec![QueryResult {
            id: format!("shell:{}", cmd),
            title: format!("> {}", cmd),
            subtitle: Some("Jalankan perintah di Windows PowerShell".to_string()),
            category: "Perintah Shell".to_string(),
            icon: ItemIcon::Symbolic("shell"),
            score: 1000,
            primary_action: Arc::new(move || {
                let _ = Command::new("powershell")
                    .args(["-NoExit", "-Command", &cmd_clone])
                    .spawn();
            }),
            secondary_action: None,
            additional_actions: Vec::new(),
        }]
    }
}
