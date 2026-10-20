use crate::core::matcher::Matcher;
use crate::core::plugin::{ItemIcon, Plugin, QueryResult};
use crate::platform::win32;
use std::sync::Arc;

const MAX_RESULTS: usize = 8;

#[derive(Clone, Copy, Debug)]
struct SettingsPage {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    uri: &'static str,
    aliases: &'static [&'static str],
}

const SETTINGS_PAGES: &[SettingsPage] = &[
    SettingsPage {
        id: "home",
        title: "Beranda Pengaturan / Settings Home",
        description: "Buka halaman utama Pengaturan Windows",
        uri: "ms-settings:",
        aliases: &["pengaturan", "setelan", "settings", "control panel"],
    },
    SettingsPage {
        id: "display",
        title: "Layar / Display",
        description: "Resolusi, skala, kecerahan, dan beberapa monitor",
        uri: "ms-settings:display",
        aliases: &[
            "layar",
            "tampilan",
            "display",
            "monitor",
            "resolusi",
            "resolution",
            "brightness",
            "kecerahan",
        ],
    },
    SettingsPage {
        id: "night-light",
        title: "Cahaya Malam / Night Light",
        description: "Atur warna hangat dan jadwal cahaya malam",
        uri: "ms-settings:nightlight",
        aliases: &["cahaya malam", "night light", "blue light", "filter biru"],
    },
    SettingsPage {
        id: "sound",
        title: "Suara / Sound",
        description: "Perangkat keluaran, masukan, volume, dan mixer audio",
        uri: "ms-settings:sound",
        aliases: &[
            "suara",
            "sound",
            "audio",
            "volume",
            "speaker",
            "mikrofon",
            "microphone",
        ],
    },
    SettingsPage {
        id: "notifications",
        title: "Notifikasi / Notifications",
        description: "Atur pemberitahuan aplikasi dan sistem",
        uri: "ms-settings:notifications",
        aliases: &[
            "notifikasi",
            "notifications",
            "pemberitahuan",
            "alerts",
            "do not disturb",
        ],
    },
    SettingsPage {
        id: "power",
        title: "Daya & Baterai / Power & Battery",
        description: "Penggunaan baterai, mode daya, layar mati, dan tidur",
        uri: "ms-settings:powersleep",
        aliases: &[
            "daya", "baterai", "power", "battery", "sleep", "tidur", "energy",
        ],
    },
    SettingsPage {
        id: "storage",
        title: "Penyimpanan / Storage",
        description: "Kelola ruang disk, file sementara, dan Storage Sense",
        uri: "ms-settings:storagesense",
        aliases: &[
            "penyimpanan",
            "storage",
            "disk",
            "ruang",
            "space",
            "storage sense",
            "file sementara",
        ],
    },
    SettingsPage {
        id: "multitasking",
        title: "Multitugas / Multitasking",
        description: "Snap windows, desktop, dan perpindahan aplikasi",
        uri: "ms-settings:multitasking",
        aliases: &["multitugas", "multitasking", "snap", "desktop", "alt tab"],
    },
    SettingsPage {
        id: "about",
        title: "Tentang PC / About",
        description: "Nama perangkat, spesifikasi, dan informasi Windows",
        uri: "ms-settings:about",
        aliases: &[
            "tentang",
            "about",
            "spesifikasi",
            "specs",
            "system info",
            "device info",
        ],
    },
    SettingsPage {
        id: "bluetooth",
        title: "Bluetooth & Perangkat / Bluetooth & Devices",
        description: "Hubungkan dan kelola perangkat Bluetooth",
        uri: "ms-settings:bluetooth",
        aliases: &[
            "bluetooth",
            "perangkat",
            "devices",
            "pair",
            "pasangkan",
            "headset",
        ],
    },
    SettingsPage {
        id: "printers",
        title: "Printer & Pemindai / Printers & Scanners",
        description: "Tambah dan kelola printer atau pemindai",
        uri: "ms-settings:printers",
        aliases: &[
            "printer", "printers", "pemindai", "scanner", "scanners", "cetak",
        ],
    },
    SettingsPage {
        id: "mouse",
        title: "Tetikus / Mouse",
        description: "Tombol utama, kecepatan penunjuk, dan gulir",
        uri: "ms-settings:mousetouchpad",
        aliases: &[
            "tetikus", "mouse", "touchpad", "trackpad", "pointer", "cursor",
        ],
    },
    SettingsPage {
        id: "typing",
        title: "Pengetikan / Typing",
        description: "Koreksi otomatis, saran teks, dan keyboard sentuh",
        uri: "ms-settings:typing",
        aliases: &[
            "pengetikan",
            "typing",
            "keyboard",
            "autocorrect",
            "saran teks",
        ],
    },
    SettingsPage {
        id: "usb",
        title: "USB",
        description: "Notifikasi dan preferensi perangkat USB",
        uri: "ms-settings:usb",
        aliases: &["usb", "flashdisk", "flash drive", "perangkat usb"],
    },
    SettingsPage {
        id: "network-status",
        title: "Status Jaringan / Network Status",
        description: "Status koneksi dan pengaturan jaringan lanjutan",
        uri: "ms-settings:network-status",
        aliases: &[
            "jaringan",
            "network",
            "internet",
            "status koneksi",
            "connection",
            "ethernet",
        ],
    },
    SettingsPage {
        id: "wifi",
        title: "Wi-Fi",
        description: "Hubungkan dan kelola jaringan nirkabel",
        uri: "ms-settings:network-wifi",
        aliases: &["wifi", "wi-fi", "wireless", "nirkabel", "hotspot network"],
    },
    SettingsPage {
        id: "ethernet",
        title: "Ethernet",
        description: "Properti dan pengaturan jaringan kabel",
        uri: "ms-settings:network-ethernet",
        aliases: &["ethernet", "lan", "kabel", "wired", "network cable"],
    },
    SettingsPage {
        id: "vpn",
        title: "VPN",
        description: "Tambah dan kelola koneksi VPN",
        uri: "ms-settings:network-vpn",
        aliases: &["vpn", "virtual private network", "jaringan privat"],
    },
    SettingsPage {
        id: "proxy",
        title: "Proksi / Proxy",
        description: "Deteksi otomatis dan server proksi manual",
        uri: "ms-settings:network-proxy",
        aliases: &["proksi", "proxy", "server proxy", "pac script"],
    },
    SettingsPage {
        id: "hotspot",
        title: "Hotspot Seluler / Mobile Hotspot",
        description: "Bagikan koneksi internet dengan perangkat lain",
        uri: "ms-settings:network-mobilehotspot",
        aliases: &[
            "hotspot",
            "mobile hotspot",
            "hotspot seluler",
            "tethering",
            "berbagi internet",
        ],
    },
    SettingsPage {
        id: "personalization",
        title: "Personalisasi / Personalization",
        description: "Latar belakang, warna, tema, dan tampilan Windows",
        uri: "ms-settings:personalization",
        aliases: &[
            "personalisasi",
            "personalization",
            "tema",
            "theme",
            "wallpaper",
            "background",
        ],
    },
    SettingsPage {
        id: "background",
        title: "Latar Belakang / Background",
        description: "Pilih gambar, warna, atau slideshow desktop",
        uri: "ms-settings:personalization-background",
        aliases: &[
            "latar belakang",
            "background",
            "wallpaper",
            "desktop picture",
        ],
    },
    SettingsPage {
        id: "colors",
        title: "Warna / Colors",
        description: "Mode terang atau gelap dan warna aksen",
        uri: "ms-settings:colors",
        aliases: &[
            "warna",
            "colors",
            "dark mode",
            "mode gelap",
            "light mode",
            "accent",
        ],
    },
    SettingsPage {
        id: "themes",
        title: "Tema / Themes",
        description: "Terapkan dan sesuaikan tema Windows",
        uri: "ms-settings:themes",
        aliases: &["tema", "themes", "theme", "skin", "tampilan desktop"],
    },
    SettingsPage {
        id: "lock-screen",
        title: "Layar Kunci / Lock Screen",
        description: "Gambar layar kunci, status, dan screensaver",
        uri: "ms-settings:lockscreen",
        aliases: &[
            "layar kunci",
            "lock screen",
            "screensaver",
            "screen saver",
            "spotlight",
        ],
    },
    SettingsPage {
        id: "taskbar",
        title: "Bilah Tugas / Taskbar",
        description: "Perilaku, ikon, dan tata letak bilah tugas",
        uri: "ms-settings:taskbar",
        aliases: &[
            "bilah tugas",
            "taskbar",
            "system tray",
            "tray",
            "ikon tugas",
        ],
    },
    SettingsPage {
        id: "apps-features",
        title: "Aplikasi Terinstal / Installed Apps",
        description: "Hapus, ubah, dan kelola aplikasi yang terinstal",
        uri: "ms-settings:appsfeatures",
        aliases: &[
            "aplikasi",
            "apps",
            "installed apps",
            "programs",
            "uninstall",
            "hapus aplikasi",
        ],
    },
    SettingsPage {
        id: "default-apps",
        title: "Aplikasi Default / Default Apps",
        description: "Pilih aplikasi bawaan untuk file dan tautan",
        uri: "ms-settings:defaultapps",
        aliases: &[
            "aplikasi default",
            "default apps",
            "file association",
            "browser bawaan",
            "default browser",
        ],
    },
    SettingsPage {
        id: "startup-apps",
        title: "Aplikasi Startup / Startup Apps",
        description: "Pilih aplikasi yang berjalan saat masuk Windows",
        uri: "ms-settings:startupapps",
        aliases: &[
            "startup",
            "startup apps",
            "aplikasi startup",
            "boot apps",
            "login apps",
        ],
    },
    SettingsPage {
        id: "accounts",
        title: "Akun / Accounts",
        description: "Kelola akun Microsoft, kerja, sekolah, dan keluarga",
        uri: "ms-settings:yourinfo",
        aliases: &[
            "akun",
            "accounts",
            "user",
            "pengguna",
            "microsoft account",
            "your info",
        ],
    },
    SettingsPage {
        id: "signin-options",
        title: "Opsi Masuk / Sign-in Options",
        description: "Windows Hello, PIN, kata sandi, dan kunci keamanan",
        uri: "ms-settings:signinoptions",
        aliases: &[
            "masuk",
            "sign in",
            "login",
            "pin",
            "password",
            "kata sandi",
            "windows hello",
            "fingerprint",
        ],
    },
    SettingsPage {
        id: "date-time",
        title: "Tanggal & Waktu / Date & Time",
        description: "Zona waktu, jam otomatis, dan sinkronisasi waktu",
        uri: "ms-settings:dateandtime",
        aliases: &[
            "tanggal",
            "waktu",
            "date",
            "time",
            "jam",
            "clock",
            "zona waktu",
            "timezone",
        ],
    },
    SettingsPage {
        id: "language",
        title: "Bahasa & Wilayah / Language & Region",
        description: "Bahasa Windows, format wilayah, dan paket bahasa",
        uri: "ms-settings:regionlanguage",
        aliases: &[
            "bahasa",
            "language",
            "wilayah",
            "region",
            "locale",
            "format tanggal",
        ],
    },
    SettingsPage {
        id: "gaming",
        title: "Gaming",
        description: "Game Bar, tangkapan, dan Mode Game",
        uri: "ms-settings:gaming-gamebar",
        aliases: &[
            "gaming",
            "game",
            "game bar",
            "xbox",
            "mode game",
            "game mode",
            "capture",
        ],
    },
    SettingsPage {
        id: "accessibility",
        title: "Aksesibilitas / Accessibility",
        description: "Penglihatan, pendengaran, interaksi, dan narator",
        uri: "ms-settings:easeofaccess",
        aliases: &[
            "aksesibilitas",
            "accessibility",
            "ease of access",
            "narator",
            "narrator",
            "magnifier",
        ],
    },
    SettingsPage {
        id: "privacy",
        title: "Privasi & Keamanan / Privacy & Security",
        description: "Izin Windows dan akses data oleh aplikasi",
        uri: "ms-settings:privacy",
        aliases: &[
            "privasi",
            "privacy",
            "keamanan",
            "security",
            "permissions",
            "izin aplikasi",
        ],
    },
    SettingsPage {
        id: "windows-update",
        title: "Pembaruan Windows / Windows Update",
        description: "Periksa, unduh, dan kelola pembaruan Windows",
        uri: "ms-settings:windowsupdate",
        aliases: &[
            "pembaruan",
            "update",
            "windows update",
            "patch",
            "check updates",
            "periksa pembaruan",
        ],
    },
    SettingsPage {
        id: "recovery",
        title: "Pemulihan / Recovery",
        description: "Reset PC, startup lanjutan, dan opsi pemulihan",
        uri: "ms-settings:recovery",
        aliases: &[
            "pemulihan",
            "recovery",
            "reset pc",
            "factory reset",
            "startup lanjutan",
        ],
    },
];

pub struct WinSettingsPlugin {
    matcher: Matcher,
}

impl WinSettingsPlugin {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
        }
    }

    fn match_score(&self, input: &str, page: &SettingsPage) -> Option<u32> {
        if input.is_empty() {
            return Some(1);
        }

        std::iter::once(page.title)
            .chain(page.aliases.iter().copied())
            .filter_map(|term| self.matcher.fuzzy_score(input, term))
            .max()
    }
}

impl Default for WinSettingsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WinSettingsPlugin {
    fn id(&self) -> &str {
        "win_settings"
    }

    fn name(&self) -> &str {
        "Windows Settings"
    }

    fn prefix(&self) -> Option<&str> {
        Some("$")
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let input = input.trim();
        let mut matches: Vec<(&SettingsPage, u32)> = SETTINGS_PAGES
            .iter()
            .filter_map(|page| self.match_score(input, page).map(|score| (page, score)))
            .collect();

        matches.sort_unstable_by(|(left_page, left_score), (right_page, right_score)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_page.id.cmp(right_page.id))
        });
        matches.truncate(MAX_RESULTS);

        matches
            .into_iter()
            .map(|(page, score)| {
                let uri = page.uri.to_string();
                let title = page.title.to_string();
                QueryResult {
                    id: format!("settings:{}", page.id),
                    title: page.title.to_string(),
                    subtitle: Some(page.description.to_string()),
                    category: "Pengaturan Windows".to_string(),
                    icon: ItemIcon::Symbolic("settings"),
                    score,
                    primary_action: Arc::new(move || {
                        if let Err(error) = open::that_detached(&uri) {
                            win32::show_error_dialog(
                                "Pengaturan Tidak Dapat Dibuka",
                                &format!(
                                    "Windows tidak dapat membuka {title}.\n\nURI: {uri}\nDetail: {error}"
                                ),
                            );
                        }
                    }),
                    secondary_action: None,
                    additional_actions: Vec::new(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_expected_plugin_metadata() {
        let plugin = WinSettingsPlugin::new();

        assert_eq!(plugin.id(), "win_settings");
        assert_eq!(plugin.name(), "Windows Settings");
        assert_eq!(plugin.prefix(), Some("$"));
        assert!(plugin.is_global());
    }

    #[test]
    fn catalog_has_unique_ids_and_valid_settings_uris() {
        let mut ids: Vec<_> = SETTINGS_PAGES.iter().map(|page| page.id).collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();

        assert!((30..=40).contains(&SETTINGS_PAGES.len()));
        assert_eq!(ids.len(), original_len);
        assert!(SETTINGS_PAGES
            .iter()
            .all(|page| page.uri.starts_with("ms-settings:")));
    }

    #[test]
    fn matches_indonesian_and_english_aliases() {
        let plugin = WinSettingsPlugin::new();

        let indonesian = plugin.query("pembaruan");
        let english = plugin.query("windows update");

        assert_eq!(indonesian[0].id, "settings:windows-update");
        assert_eq!(english[0].id, "settings:windows-update");
    }

    #[test]
    fn fuzzy_results_are_ranked_and_limited_deterministically() {
        let plugin = WinSettingsPlugin::new();
        let first = plugin.query("");
        let second = plugin.query("");

        assert_eq!(first.len(), MAX_RESULTS);
        assert_eq!(
            first.iter().map(|result| &result.id).collect::<Vec<_>>(),
            second.iter().map(|result| &result.id).collect::<Vec<_>>()
        );
        assert!(first.windows(2).all(|pair| pair[0].id < pair[1].id));
    }

    #[test]
    fn result_metadata_identifies_windows_settings() {
        let plugin = WinSettingsPlugin::new();
        let result = plugin.query("bluetooth").remove(0);

        assert_eq!(result.id, "settings:bluetooth");
        assert_eq!(result.category, "Pengaturan Windows");
        assert_eq!(result.icon, ItemIcon::Symbolic("settings"));
        assert!(result.subtitle.is_some());
    }
}
