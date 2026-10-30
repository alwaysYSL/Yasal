use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::platform::win32;
use crate::plugins::app_launcher::set_clipboard_text;
use std::io::{BufRead, BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Id prefix of the command row. Its secondary action runs the command in place and keeps
/// the palette open, so the output can be shown.
pub const RUN_INLINE_ID_PREFIX: &str = "shell:cmd:";

/// Longest a command may run before it is stopped.
const RUN_TIMEOUT: Duration = Duration::from_secs(15);
/// Lines kept from the output. Extra lines are counted but not shown.
const MAX_OUTPUT_LINES: usize = 200;
/// Characters shown in an output row. The full line is still copied.
const MAX_TITLE_CHARS: usize = 120;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Makes PowerShell write UTF-8, so non-ASCII output is not garbled.
const UTF8_OUTPUT_PREFIX: &str =
    "try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch { }; ";

#[derive(Clone, Debug, PartialEq, Eq)]
enum RunState {
    Running,
    Finished {
        exit_code: Option<i32>,
        lines: Vec<String>,
        total_lines: usize,
        timed_out: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShellRun {
    command: String,
    state: RunState,
}

/// The latest run. Only one run is kept, because the palette shows one command at a time.
type SharedRun = Arc<Mutex<Option<ShellRun>>>;

pub struct ShellCommandPlugin {
    run: SharedRun,
}

impl ShellCommandPlugin {
    pub fn new() -> Self {
        Self {
            run: Arc::new(Mutex::new(None)),
        }
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
        let command = input.trim_start_matches('>').trim();
        if command.is_empty() {
            return Vec::new();
        }

        let mut results = vec![command_result(command, &self.run)];

        let latest = self
            .run
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
            .filter(|run| run.command == command);
        if let Some(run) = latest {
            results.push(status_result(command, &run.state));
            if let RunState::Finished { lines, .. } = &run.state {
                results.extend(
                    lines
                        .iter()
                        .enumerate()
                        .map(|(index, line)| output_result(index, line)),
                );
            }
        }
        results
    }
}

fn open_in_powershell(command: &str) {
    if Command::new("powershell")
        .args(["-NoExit", "-Command", command])
        .spawn()
        .is_err()
    {
        win32::show_error_dialog(
            "PowerShell Tidak Dapat Dibuka",
            "Windows tidak dapat menjalankan perintah di PowerShell.",
        );
    }
}

fn command_result(command: &str, run: &SharedRun) -> QueryResult {
    let open_command = command.to_string();
    let inline_action = ItemAction {
        label: "Jalankan dan tampilkan output".to_string(),
        icon: "shell",
        shortcut_hint: Some("Ctrl+Enter".to_string()),
        action: start_run(command.to_string(), Arc::clone(run)),
    };

    QueryResult {
        id: format!("{RUN_INLINE_ID_PREFIX}{command}"),
        title: format!("> {command}"),
        subtitle: Some(
            "Enter: buka di PowerShell · Ctrl+Enter: jalankan dan tampilkan output".to_string(),
        ),
        category: "Perintah Shell".to_string(),
        icon: ItemIcon::Symbolic("shell"),
        score: 1_000,
        primary_action: Arc::new(move || open_in_powershell(&open_command)),
        secondary_action: Some(inline_action),
        additional_actions: Vec::new(),
    }
}

/// Starts the command on a worker thread and returns at once. The palette polls the
/// shared state until the run finishes.
fn start_run(command: String, run: SharedRun) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        {
            let Ok(mut guard) = run.lock() else {
                return;
            };
            let already_running = guard.as_ref().is_some_and(|current| {
                current.command == command && current.state == RunState::Running
            });
            if already_running {
                return;
            }
            *guard = Some(ShellRun {
                command: command.clone(),
                state: RunState::Running,
            });
        }

        let run = Arc::clone(&run);
        let command = command.clone();
        std::thread::spawn(move || {
            let state = capture_output(&command);
            if let Ok(mut guard) = run.lock() {
                if let Some(current) = guard.as_mut() {
                    // Ignore the result when a different command has replaced this run.
                    if current.command == command {
                        current.state = state;
                    }
                }
            }
        });
    })
}

/// Runs `command` in PowerShell with no window, collecting stdout and stderr.
fn capture_output(command: &str) -> RunState {
    let script = format!("{UTF8_OUTPUT_PREFIX}{command}");
    let spawned = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            return RunState::Finished {
                exit_code: None,
                lines: vec![format!("PowerShell tidak dapat dijalankan: {error}")],
                total_lines: 1,
                timed_out: false,
            };
        }
    };

    // Both pipes are read by their own threads and sent through one bounded channel. The
    // bound makes a noisy command wait on this thread instead of filling memory.
    let (sender, receiver) = mpsc::sync_channel::<String>(256);
    if let Some(stdout) = child.stdout.take() {
        let sender = sender.clone();
        std::thread::spawn(move || forward_lines(stdout, sender));
    }
    if let Some(stderr) = child.stderr.take() {
        let sender = sender.clone();
        std::thread::spawn(move || forward_lines(stderr, sender));
    }
    drop(sender);

    let mut collected = OutputCollector::default();
    let deadline = Instant::now() + RUN_TIMEOUT;
    let (exit_code, timed_out) = loop {
        collected.drain(&receiver);
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(15)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                break (None, true);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break (None, false);
            }
        }
    };

    // Lines written just before exit can still be in the pipes. A grandchild process may
    // keep a pipe open after PowerShell is gone, so this wait is bounded.
    collected.drain_until_closed(&receiver, Duration::from_millis(300));

    RunState::Finished {
        exit_code,
        lines: collected.lines,
        total_lines: collected.total,
        timed_out,
    }
}

fn forward_lines(reader: impl Read, sender: mpsc::SyncSender<String>) {
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let line = String::from_utf8_lossy(&buffer)
            .trim_end_matches(['\r', '\n'])
            .to_string();
        if line.trim().is_empty() {
            continue;
        }
        if sender.send(line).is_err() {
            break;
        }
    }
}

#[derive(Default)]
struct OutputCollector {
    lines: Vec<String>,
    total: usize,
}

impl OutputCollector {
    fn push(&mut self, line: String) {
        self.total += 1;
        if self.lines.len() < MAX_OUTPUT_LINES {
            self.lines.push(line);
        }
    }

    fn drain(&mut self, receiver: &mpsc::Receiver<String>) {
        while let Ok(line) = receiver.try_recv() {
            self.push(line);
        }
    }

    fn drain_until_closed(&mut self, receiver: &mpsc::Receiver<String>, grace: Duration) {
        let deadline = Instant::now() + grace;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(line) => self.push(line),
                Err(_) => break,
            }
        }
    }
}

/// One line for the palette, with an ellipsis when it is longer than `max_chars`.
fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let head: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{head}…")
}

/// Row shown while the command runs, and after it finishes with a summary.
fn status_result(command: &str, state: &RunState) -> QueryResult {
    match state {
        RunState::Running => {
            let open_command = command.to_string();
            QueryResult {
                id: "shell:running".to_string(),
                title: "Menjalankan perintah…".to_string(),
                subtitle: Some(
                    "Output muncul di sini setelah selesai · Enter: buka di PowerShell".to_string(),
                ),
                category: "Perintah Shell".to_string(),
                icon: ItemIcon::Symbolic("shell"),
                score: 999,
                primary_action: Arc::new(move || open_in_powershell(&open_command)),
                secondary_action: None,
                additional_actions: Vec::new(),
            }
        }
        RunState::Finished {
            exit_code,
            lines,
            total_lines,
            timed_out,
        } => {
            let outcome = if *timed_out {
                format!("Dihentikan: lebih dari {} detik", RUN_TIMEOUT.as_secs())
            } else {
                match exit_code {
                    Some(code) => format!("Selesai · exit {code}"),
                    None => "Selesai".to_string(),
                }
            };
            let count = if *total_lines > lines.len() {
                format!("{total_lines} baris ({} ditampilkan)", lines.len())
            } else {
                format!("{total_lines} baris")
            };

            let (subtitle, primary_action): (String, Arc<dyn Fn() + Send + Sync>) =
                if lines.is_empty() {
                    let open_command = command.to_string();
                    (
                        "Tidak ada output · Enter: buka di PowerShell".to_string(),
                        Arc::new(move || open_in_powershell(&open_command)),
                    )
                } else {
                    let all_output = lines.join("\n");
                    (
                        "Enter: salin seluruh output".to_string(),
                        Arc::new(move || {
                            let _ = set_clipboard_text(&all_output);
                        }),
                    )
                };

            QueryResult {
                id: "shell:done".to_string(),
                title: format!("{outcome} · {count}"),
                subtitle: Some(subtitle),
                category: "Perintah Shell".to_string(),
                icon: ItemIcon::Symbolic("shell"),
                score: 999,
                primary_action,
                secondary_action: None,
                additional_actions: Vec::new(),
            }
        }
    }
}

fn output_result(index: usize, line: &str) -> QueryResult {
    let text = line.to_string();
    QueryResult {
        id: format!("shell:out:{index}"),
        title: truncate_chars(line, MAX_TITLE_CHARS),
        subtitle: None,
        category: "Perintah Shell".to_string(),
        icon: ItemIcon::Symbolic("shell"),
        score: 500,
        primary_action: Arc::new(move || {
            let _ = set_clipboard_text(&text);
        }),
        secondary_action: None,
        additional_actions: Vec::new(),
    }
}
