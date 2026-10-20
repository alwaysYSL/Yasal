use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::plugins::app_launcher::set_clipboard_text;
use std::net::Ipv4Addr;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NetworkSnapshot {
    local_ipv4: Option<String>,
    mac_address: Option<String>,
    default_gateway: Option<String>,
    public_ip: Option<String>,
}

#[derive(Clone, Debug)]
enum NetworkState {
    Idle,
    Loading,
    Ready(NetworkSnapshot),
    Unavailable(String),
}

pub struct NetworkInfoPlugin {
    cache: Arc<RwLock<NetworkCache>>,
}

#[derive(Clone, Debug)]
struct NetworkCache {
    state: NetworkState,
    last_refresh: Option<Instant>,
}

impl NetworkInfoPlugin {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(NetworkCache {
                state: NetworkState::Idle,
                last_refresh: None,
            })),
        }
    }

    fn ensure_fresh(&self) {
        let now = Instant::now();
        let Ok(mut cache) = self.cache.write() else {
            return;
        };
        if !refresh_due(&cache, now) {
            return;
        }
        cache.state = NetworkState::Loading;
        cache.last_refresh = Some(now);
        drop(cache);

        let worker_cache = Arc::clone(&self.cache);

        if let Err(error) = std::thread::Builder::new()
            .name("network-info".to_string())
            .spawn(move || {
                let next_state = match collect_network_info() {
                    Ok(snapshot) if snapshot.local_ipv4.is_some() => NetworkState::Ready(snapshot),
                    Ok(_) => NetworkState::Unavailable(
                        "No active IPv4 network adapter was found".to_string(),
                    ),
                    Err(error) => NetworkState::Unavailable(error),
                };

                if let Ok(mut cache) = worker_cache.write() {
                    cache.state = next_state;
                }
            })
        {
            if let Ok(mut cache) = self.cache.write() {
                cache.state = NetworkState::Unavailable(format!(
                    "Could not start the network information worker: {error}"
                ));
            }
        }
    }

    fn is_trigger(input: &str) -> bool {
        matches!(input.trim().to_ascii_lowercase().as_str(), "ip" | "network")
    }

    fn value_result(
        id: &str,
        title: &str,
        value: &str,
        description: &str,
        score: u32,
    ) -> QueryResult {
        let value_to_copy = value.to_string();
        let copy_action: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            let _ = set_clipboard_text(&value_to_copy);
        });

        QueryResult {
            id: id.to_string(),
            title: format!("{title}: {value}"),
            subtitle: Some(format!("{description} · Tekan Enter untuk menyalin")),
            category: "Jaringan".to_string(),
            icon: ItemIcon::Symbolic("network"),
            score,
            primary_action: Arc::clone(&copy_action),
            secondary_action: Some(ItemAction {
                label: format!("Salin {title}"),
                icon: "📋",
                shortcut_hint: Some("Enter / Ctrl+Enter".to_string()),
                action: copy_action,
            }),
            additional_actions: Vec::new(),
        }
    }

    fn status_result(title: &str, subtitle: String, score: u32) -> QueryResult {
        QueryResult {
            id: "network:status".to_string(),
            title: title.to_string(),
            subtitle: Some(subtitle),
            category: "Jaringan".to_string(),
            icon: ItemIcon::Symbolic("network"),
            score,
            primary_action: Arc::new(|| {}),
            secondary_action: None,
            additional_actions: Vec::new(),
        }
    }
}

impl Default for NetworkInfoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NetworkInfoPlugin {
    fn id(&self) -> &str {
        "network_info"
    }

    fn name(&self) -> &str {
        "Network Info"
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        if !Self::is_trigger(input) {
            return Vec::new();
        }

        self.ensure_fresh();

        let state = match self.cache.read() {
            Ok(cache) => cache.state.clone(),
            Err(_) => {
                return vec![Self::status_result(
                    "Informasi jaringan tidak tersedia",
                    "Cache jaringan tidak dapat dibaca.".to_string(),
                    900,
                )];
            }
        };

        match state {
            NetworkState::Idle | NetworkState::Loading => vec![Self::status_result(
                "Memuat informasi jaringan lokal…",
                "IPv4 lokal dan IP publik dikumpulkan di background.".to_string(),
                900,
            )],
            NetworkState::Unavailable(reason) => vec![Self::status_result(
                "Informasi jaringan tidak tersedia",
                reason,
                900,
            )],
            NetworkState::Ready(snapshot) => {
                let mut results = Vec::with_capacity(4);

                if let Some(ipv4) = snapshot.local_ipv4 {
                    results.push(Self::value_result(
                        "network:local-ipv4",
                        "IPv4 lokal",
                        &ipv4,
                        "Address on the active Windows network adapter",
                        1_000,
                    ));
                }
                if let Some(gateway) = snapshot.default_gateway {
                    results.push(Self::value_result(
                        "network:gateway",
                        "Gateway bawaan",
                        &gateway,
                        "Router used by the active network adapter",
                        980,
                    ));
                }
                if let Some(mac) = snapshot.mac_address {
                    results.push(Self::value_result(
                        "network:mac",
                        "Alamat MAC",
                        &mac,
                        "Hardware address of the active network adapter",
                        960,
                    ));
                }

                if let Some(public_ip) = snapshot.public_ip {
                    results.push(Self::value_result(
                        "network:public-ip",
                        "IP publik",
                        &public_ip,
                        "Alamat internet publik dari api.ipify.org",
                        940,
                    ));
                } else {
                    results.push(Self::status_result(
                        "IP publik tidak tersedia",
                        "Permintaan latar belakang gagal atau koneksi sedang offline.".to_string(),
                        850,
                    ));
                }
                results
            }
        }
    }
}

fn refresh_due(cache: &NetworkCache, now: Instant) -> bool {
    !matches!(cache.state, NetworkState::Loading)
        && cache
            .last_refresh
            .is_none_or(|then| now.duration_since(then) >= CACHE_TTL)
}

fn collect_network_info() -> Result<NetworkSnapshot, String> {
    let output = Command::new("ipconfig")
        .arg("/all")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("Could not run ipconfig: {error}"))?;

    if !output.status.success() {
        return Err(format!("ipconfig exited with status {}", output.status));
    }

    let mut snapshot = parse_ipconfig(&String::from_utf8_lossy(&output.stdout));
    snapshot.public_ip = fetch_public_ip();
    Ok(snapshot)
}

fn fetch_public_ip() -> Option<String> {
    let output = Command::new("curl.exe")
        .args([
            "--silent",
            "--show-error",
            "--connect-timeout",
            "1",
            "--max-time",
            "2",
            "https://api.ipify.org",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    value.parse::<std::net::IpAddr>().ok().map(|_| value)
}

fn parse_ipconfig(output: &str) -> NetworkSnapshot {
    let mut adapters = Vec::new();
    let mut current = NetworkSnapshot::default();
    let mut awaiting_gateway = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            awaiting_gateway = false;
            continue;
        }

        if !line.starts_with([' ', '\t']) && trimmed.ends_with(':') {
            if current.local_ipv4.is_some() {
                adapters.push(current);
            }
            current = NetworkSnapshot::default();
            awaiting_gateway = false;
            continue;
        }

        let lowercase = trimmed.to_ascii_lowercase();
        let value = property_value(trimmed);

        if lowercase.contains("ipv4") {
            if let Some(address) = value.and_then(parse_usable_ipv4) {
                current.local_ipv4 = Some(address.to_string());
            }
        }

        if current.mac_address.is_none() {
            if let Some(mac) = value.and_then(parse_mac_address) {
                current.mac_address = Some(mac);
            }
        }

        let is_gateway_property = lowercase.contains("default gateway")
            || lowercase.contains("gateway default")
            || lowercase.contains("gateway baku");
        if is_gateway_property {
            current.default_gateway = value
                .and_then(parse_usable_ipv4)
                .map(|address| address.to_string());
            awaiting_gateway = current.default_gateway.is_none();
        } else if awaiting_gateway {
            if let Some(address) = parse_usable_ipv4(trimmed) {
                current.default_gateway = Some(address.to_string());
            }
            awaiting_gateway = false;
        }
    }

    if current.local_ipv4.is_some() {
        adapters.push(current);
    }

    adapters
        .iter()
        .find(|adapter| adapter.default_gateway.is_some())
        .or_else(|| adapters.first())
        .cloned()
        .unwrap_or_default()
}

fn property_value(line: &str) -> Option<&str> {
    line.split_once(':')
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
}

fn parse_usable_ipv4(value: &str) -> Option<Ipv4Addr> {
    value
        .split_whitespace()
        .map(|part| {
            part.split(['(', '['])
                .next()
                .unwrap_or(part)
                .trim_matches(|character: char| matches!(character, ')' | ']' | ',' | ';'))
        })
        .find_map(|part| part.parse::<Ipv4Addr>().ok())
        .filter(|address| {
            !address.is_loopback()
                && !address.is_link_local()
                && !address.is_unspecified()
                && !address.is_broadcast()
        })
}

fn parse_mac_address(value: &str) -> Option<String> {
    value.split_whitespace().find_map(|part| {
        let candidate = part.trim_matches(|character: char| {
            !character.is_ascii_hexdigit() && character != '-' && character != ':'
        });
        let octets: Vec<&str> = candidate.split(['-', ':']).collect();
        let valid = octets.len() == 6
            && octets
                .iter()
                .all(|octet| octet.len() == 2 && octet.chars().all(|c| c.is_ascii_hexdigit()));

        valid.then(|| octets.join(":").to_ascii_uppercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_collection_is_lazy_and_cache_expires_only_on_query() {
        let plugin = NetworkInfoPlugin::new();
        let cache = plugin.cache.read().unwrap();
        assert!(matches!(cache.state, NetworkState::Idle));
        assert!(cache.last_refresh.is_none());
        drop(cache);

        let now = Instant::now();
        let ready = NetworkCache {
            state: NetworkState::Ready(NetworkSnapshot::default()),
            last_refresh: Some(now),
        };
        assert!(!refresh_due(
            &ready,
            now + CACHE_TTL - Duration::from_millis(1)
        ));
        assert!(refresh_due(&ready, now + CACHE_TTL));
        let loading = NetworkCache {
            state: NetworkState::Loading,
            last_refresh: None,
        };
        assert!(!refresh_due(&loading, now));
    }

    #[test]
    fn triggers_only_for_explicit_global_queries() {
        assert!(NetworkInfoPlugin::is_trigger("ip"));
        assert!(NetworkInfoPlugin::is_trigger(" Network "));
        assert!(NetworkInfoPlugin::is_trigger("IP"));
        assert!(!NetworkInfoPlugin::is_trigger(""));
        assert!(!NetworkInfoPlugin::is_trigger("iphone"));
        assert!(!NetworkInfoPlugin::is_trigger("network settings"));
    }

    #[test]
    fn parses_values_from_the_same_active_adapter() {
        let output = r#"
Windows IP Configuration

Ethernet adapter Disconnected:

   Physical Address. . . . . . . . . : 00-11-22-33-44-55
   Media State . . . . . . . . . . . : Media disconnected

Wireless LAN adapter Wi-Fi:

   Physical Address. . . . . . . . . : aa-bb-cc-dd-ee-ff
   IPv4 Address. . . . . . . . . . . : 192.168.1.42(Preferred)
   Default Gateway . . . . . . . . . : 192.168.1.1
"#;

        assert_eq!(
            parse_ipconfig(output),
            NetworkSnapshot {
                local_ipv4: Some("192.168.1.42".to_string()),
                mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
                default_gateway: Some("192.168.1.1".to_string()),
                public_ip: None,
            }
        );
    }

    #[test]
    fn prefers_adapter_with_a_gateway_and_parses_continuation() {
        let output = r#"
Ethernet adapter Virtual:
   Physical Address. . . . . . . . . : 10-20-30-40-50-60
   IPv4 Address. . . . . . . . . . . : 10.10.0.2(Preferred)

Ethernet adapter Main:
   Physical Address. . . . . . . . . : 01:23:45:67:89:ab
   IPv4 Address. . . . . . . . . . . : 172.16.0.5(Preferred)
   Default Gateway . . . . . . . . . :
                                       172.16.0.1
"#;

        let parsed = parse_ipconfig(output);
        assert_eq!(parsed.local_ipv4.as_deref(), Some("172.16.0.5"));
        assert_eq!(parsed.mac_address.as_deref(), Some("01:23:45:67:89:AB"));
        assert_eq!(parsed.default_gateway.as_deref(), Some("172.16.0.1"));
    }

    #[test]
    fn rejects_loopback_apipa_and_malformed_mac_values() {
        assert_eq!(parse_usable_ipv4("127.0.0.1"), None);
        assert_eq!(parse_usable_ipv4("169.254.2.3"), None);
        assert_eq!(parse_usable_ipv4("0.0.0.0"), None);
        assert_eq!(parse_mac_address("01-23-45-67-89"), None);
        assert_eq!(
            parse_mac_address("value: de-ad-be-ef-00-01"),
            Some("DE:AD:BE:EF:00:01".to_string())
        );
    }
}
