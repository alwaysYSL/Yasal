use crate::core::matcher::Matcher;
use crate::core::plugin::{ItemIcon, Plugin, QueryResult};
use crate::platform::win32;
use std::sync::Arc;

pub struct SystemCommandsPlugin {
    matcher: Matcher,
}

impl SystemCommandsPlugin {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
        }
    }
}

impl Default for SystemCommandsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SystemCommandsPlugin {
    fn id(&self) -> &str {
        "system_commands"
    }

    fn name(&self) -> &str {
        "System Commands"
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let mut results = Vec::new();

        let commands = [
            (
                "sys:lock",
                "Lock Screen / Kunci Layar",
                "Kunci sesi Windows saat ini",
                "lock",
                Arc::new(|| {
                    win32::lock_workstation();
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
            (
                "sys:sleep",
                "Sleep / Mode Tidur",
                "Masukkan komputer ke mode hemat daya tidur",
                "moon",
                Arc::new(|| {
                    win32::sleep_system();
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
            (
                "sys:recycle",
                "Empty Recycle Bin / Kosongkan Keranjang Sampah",
                "Hapus seluruh file yang ada di Recycle Bin secara permanen",
                "trash",
                Arc::new(|| {
                    win32::empty_recycle_bin();
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
            (
                "sys:restart",
                "Restart / Mulai Ulang Komputer",
                "Reboot sistem Windows (dengan konfirmasi)",
                "power",
                Arc::new(|| {
                    if win32::show_confirm_dialog(
                        "Konfirmasi Restart",
                        "Apakah Anda yakin ingin me-restart komputer?",
                    ) {
                        if let Err(error) = win32::restart_system_result() {
                            win32::show_error_dialog(
                                "Restart Gagal",
                                &format!(
                                    "Windows tidak dapat me-restart komputer.\n\nDetail: {error}\n\nPastikan akun Anda memiliki izin untuk me-restart Windows, lalu coba lagi.",
                                ),
                            );
                        }
                    }
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
            (
                "sys:shutdown",
                "Shutdown / Matikan Komputer",
                "Matikan daya komputer sepenuhnya (dengan konfirmasi)",
                "power",
                Arc::new(|| {
                    if win32::show_confirm_dialog(
                        "Konfirmasi Shutdown",
                        "Apakah Anda yakin ingin mematikan komputer?",
                    ) {
                        if let Err(error) = win32::shutdown_system_result() {
                            win32::show_error_dialog(
                                "Shutdown Gagal",
                                &format!(
                                    "Windows tidak dapat mematikan komputer.\n\nDetail: {error}\n\nPastikan akun Anda memiliki izin untuk mematikan Windows, lalu coba lagi.",
                                ),
                            );
                        }
                    }
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
            (
                "sys:signout",
                "Sign Out / Keluar Akun",
                "Keluar dari sesi akun pengguna saat ini",
                "power",
                Arc::new(|| {
                    if win32::show_confirm_dialog(
                        "Konfirmasi Sign Out",
                        "Apakah Anda yakin ingin keluar dari akun?",
                    ) {
                        if let Err(error) = win32::sign_out_system_result() {
                            win32::show_error_dialog(
                                "Sign Out Gagal",
                                &format!(
                                    "Windows tidak dapat mengeluarkan akun saat ini.\n\nDetail: {error}\n\nTutup aplikasi yang memblokir proses keluar, lalu coba lagi.",
                                ),
                            );
                        }
                    }
                }) as Arc<dyn Fn() + Send + Sync>,
            ),
        ];

        for (id, title, subtitle, icon, action) in commands {
            let score = if input.is_empty() {
                Some(5)
            } else {
                self.matcher.fuzzy_score(input, title)
            };

            if let Some(score) = score {
                results.push(QueryResult {
                    id: id.to_string(),
                    title: title.to_string(),
                    subtitle: Some(subtitle.to_string()),
                    category: "Sistem".to_string(),
                    icon: ItemIcon::Symbolic(icon),
                    score,
                    primary_action: action,
                    secondary_action: None,
                    additional_actions: Vec::new(),
                });
            }
        }

        results
    }
}
