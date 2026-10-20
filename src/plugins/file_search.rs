use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::platform::win32;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_RESULTS: usize = 8;
const MAX_EVERYTHING_RESULTS: usize = 64;
const MAX_INDEX_ENTRIES: usize = 15_000;
const MAX_INDEX_PATH_BYTES: usize = 4 * 1024 * 1024;
const EVERYTHING_TIMEOUT: Duration = Duration::from_millis(250);
const SEARCH_DEBOUNCE: Duration = Duration::from_millis(150);
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug)]
struct FileEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

impl FileEntry {
    fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        if name.is_empty() {
            return None;
        }

        let is_dir = path.is_dir();
        Some(Self { path, name, is_dir })
    }
}

pub struct FileSearchPlugin {
    everything: Option<PathBuf>,
    index: Arc<OnceLock<Vec<FileEntry>>>,
    search: Arc<Mutex<SearchState>>,
    worker_tx: Mutex<Option<mpsc::Sender<String>>>,
}

#[derive(Default)]
struct SearchState {
    requested: String,
    completed: String,
    entries: Vec<FileEntry>,
    loading: bool,
}

impl FileSearchPlugin {
    pub fn new() -> Self {
        Self {
            everything: find_everything_cli(),
            index: Arc::new(OnceLock::new()),
            search: Arc::new(Mutex::new(SearchState::default())),
            worker_tx: Mutex::new(None),
        }
    }

    fn ensure_worker(&self) -> Option<mpsc::Sender<String>> {
        let mut sender = self.worker_tx.lock().ok()?;
        if let Some(sender) = sender.as_ref() {
            return Some(sender.clone());
        }
        let (tx, rx) = mpsc::channel();
        let search = Arc::clone(&self.search);
        let index = Arc::clone(&self.index);
        let everything = self.everything.clone();
        std::thread::Builder::new()
            .name("yasal-file-search".to_string())
            .spawn(move || file_search_worker(rx, search, index, everything))
            .ok()?;
        *sender = Some(tx.clone());
        Some(tx)
    }

    #[cfg(test)]
    fn with_mock_index(paths: &[&str]) -> Self {
        let index = OnceLock::new();
        let entries = paths
            .iter()
            .filter_map(|path| FileEntry::from_path(PathBuf::from(path)))
            .collect();
        let _ = index.set(entries);

        Self {
            everything: None,
            index: Arc::new(index),
            search: Arc::new(Mutex::new(SearchState::default())),
            worker_tx: Mutex::new(None),
        }
    }
}

fn file_search_worker(
    rx: mpsc::Receiver<String>,
    search: Arc<Mutex<SearchState>>,
    index: Arc<OnceLock<Vec<FileEntry>>>,
    everything: Option<PathBuf>,
) {
    while let Ok(mut query) = rx.recv() {
        loop {
            match rx.recv_timeout(SEARCH_DEBOUNCE) {
                Ok(newer) => query = newer,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        let entries = search_everything(everything.as_ref(), &query).unwrap_or_else(|| {
            let entries = index.get_or_init(build_fallback_index);
            rank_entries(entries.iter(), &query)
        });
        if let Ok(mut state) = search.lock() {
            if state.requested == query {
                state.completed = query;
                state.entries = entries;
                state.loading = false;
            }
        }
    }
}

fn search_everything(executable: Option<&PathBuf>, query: &str) -> Option<Vec<FileEntry>> {
    let executable = executable?;
    let mut child = Command::new(executable)
        .arg("-utf8")
        .arg("-n")
        .arg(MAX_EVERYTHING_RESULTS.to_string())
        .arg(query)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .ok()?;

    // Drain stdout while the process runs; waiting for exit before reading
    // can deadlock when 64 long paths fill the pipe buffer.
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        stdout.read_to_end(&mut output).ok()?;
        Some(output)
    });

    let deadline = Instant::now() + EVERYTHING_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return None;
            }
        }
    };

    if !status.success() {
        return None;
    }

    let output = reader.join().ok()??;
    let entries = String::from_utf8_lossy(&output)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(MAX_EVERYTHING_RESULTS)
        .filter_map(|line| FileEntry::from_path(PathBuf::from(line.trim_end())))
        .collect::<Vec<_>>();

    Some(rank_entries(entries.iter(), query))
}

impl Default for FileSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for FileSearchPlugin {
    fn id(&self) -> &str {
        "file_search"
    }

    fn name(&self) -> &str {
        "File Search"
    }

    fn prefix(&self) -> Option<&str> {
        Some("file")
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let query = input.trim();
        if query.is_empty() {
            return Vec::new();
        }

        let Ok(mut state) = self.search.lock() else {
            return Vec::new();
        };
        if state.requested != query {
            state.requested = query.to_string();
            state.loading = true;
            if self
                .ensure_worker()
                .is_none_or(|tx| tx.send(query.to_string()).is_err())
            {
                state.loading = false;
                return Vec::new();
            }
        }
        if state.loading || state.completed != query {
            return vec![indexing_result()];
        }
        let entries = state.entries.clone();
        drop(state);

        entries
            .into_iter()
            .take(MAX_RESULTS)
            .map(entry_to_result)
            .collect()
    }
}

fn indexing_result() -> QueryResult {
    QueryResult {
        id: "file:indexing".to_string(),
        title: "Mencari file…".to_string(),
        subtitle: Some("Everything atau indeks folder sedang diproses di background".to_string()),
        category: "File".to_string(),
        icon: ItemIcon::Symbolic("file"),
        score: 1_000,
        primary_action: Arc::new(|| {}),
        secondary_action: None,
        additional_actions: Vec::new(),
    }
}

fn entry_to_result(entry: FileEntry) -> QueryResult {
    let path_text = entry.path.to_string_lossy().into_owned();
    let title = entry.name;
    let category = if entry.is_dir { "Folder" } else { "File" };
    let primary_path = entry.path.clone();
    let reveal_path = entry.path;

    let primary_action = Arc::new(move || {
        if open::that_detached(&primary_path).is_err() {
            win32::show_error_dialog(
                "File Tidak Dapat Dibuka",
                "File atau folder tidak tersedia atau tidak memiliki aplikasi pembuka.",
            );
        }
    });

    let secondary_action = ItemAction {
        label: "Tampilkan di Explorer".to_string(),
        icon: "📁",
        shortcut_hint: None,
        action: Arc::new(move || {
            if Command::new("explorer.exe")
                .arg("/select,")
                .arg(&reveal_path)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .is_err()
            {
                win32::show_error_dialog(
                    "Explorer Tidak Dapat Dibuka",
                    "Windows Explorer tidak dapat menampilkan lokasi file atau folder.",
                );
            }
        }),
    };

    QueryResult {
        id: format!("file:{}", path_text.to_lowercase()),
        title,
        subtitle: Some(path_text.clone()),
        category: category.to_string(),
        icon: ItemIcon::File(path_text),
        score: 900,
        primary_action,
        secondary_action: Some(secondary_action.clone()),
        additional_actions: vec![secondary_action],
    }
}

fn rank_entries<'a>(entries: impl Iterator<Item = &'a FileEntry>, query: &str) -> Vec<FileEntry> {
    let query = query.to_lowercase();
    let mut matches = entries
        .filter_map(|entry| match_score(&query, entry).map(|score| (score, entry)))
        .collect::<Vec<_>>();

    matches.sort_unstable_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.path.cmp(&right.path))
    });
    matches.truncate(MAX_RESULTS);
    matches
        .into_iter()
        .map(|(_, entry)| entry.clone())
        .collect()
}

fn match_score(query: &str, entry: &FileEntry) -> Option<u32> {
    let name = entry.name.to_lowercase();
    let path = entry.path.to_string_lossy().to_lowercase();

    if name == query {
        return Some(1_200);
    }
    if entry
        .path
        .file_stem()
        .is_some_and(|stem| stem.to_string_lossy().to_lowercase() == query)
    {
        return Some(1_150);
    }
    if name.starts_with(query) {
        return Some(1_050);
    }
    if let Some(position) = name.find(query) {
        return Some(900_u32.saturating_sub(position as u32));
    }
    if let Some(position) = path.find(query) {
        return Some(700_u32.saturating_sub(position as u32));
    }

    subsequence_score(query, &name)
        .map(|score| score + 400)
        .or_else(|| subsequence_score(query, &path).map(|score| score + 100))
}

fn subsequence_score(query: &str, candidate: &str) -> Option<u32> {
    let mut wanted = query.chars();
    let mut current = wanted.next()?;
    let mut matched = 0_u32;
    let mut gaps = 0_u32;
    let mut previous_match = None;

    for (position, character) in candidate.chars().enumerate() {
        if character == current {
            if let Some(previous) = previous_match {
                gaps = gaps.saturating_add(position.saturating_sub(previous + 1) as u32);
            }
            previous_match = Some(position);
            matched += 1;
            match wanted.next() {
                Some(next) => current = next,
                None => return Some(matched.saturating_mul(20).saturating_sub(gaps)),
            }
        }
    }

    None
}

fn find_everything_cli() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join("es.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .flat_map(|base| {
            [
                base.join(r"Everything\es.exe"),
                base.join(r"voidtools\Everything\es.exe"),
            ]
        })
        .find(|path| path.is_file())
}

fn build_fallback_index() -> Vec<FileEntry> {
    let mut roots = Vec::new();
    if let Some(path) = dirs::document_dir() {
        roots.push(path);
    }
    if let Some(path) = dirs::desktop_dir() {
        roots.push(path);
    }
    if let Some(path) = dirs::download_dir() {
        roots.push(path);
    }

    let mut seen_roots = HashSet::new();
    roots.retain(|path| seen_roots.insert(path.clone()));

    let mut entries = Vec::new();
    let mut stack = roots;
    let mut path_bytes = 0_usize;

    while let Some(directory) = stack.pop() {
        let Ok(children) = fs::read_dir(directory) else {
            continue;
        };

        for child in children.flatten() {
            if entries.len() >= MAX_INDEX_ENTRIES || path_bytes >= MAX_INDEX_PATH_BYTES {
                return entries;
            }

            let Ok(file_type) = child.file_type() else {
                continue;
            };
            let path = child.path();
            let path_size = path.as_os_str().to_string_lossy().len();
            if path_bytes.saturating_add(path_size) > MAX_INDEX_PATH_BYTES {
                return entries;
            }

            let name = child.file_name().to_string_lossy().into_owned();
            if name.is_empty() {
                continue;
            }

            let is_dir = file_type.is_dir();
            if is_dir {
                stack.push(path.clone());
            }
            path_bytes += path_size;
            entries.push(FileEntry { path, name, is_dir });
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn await_results(plugin: &FileSearchPlugin, query: &str) -> Vec<QueryResult> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let results = plugin.query(query);
            if !results.iter().any(|result| result.id == "file:indexing") {
                return results;
            }
            assert!(Instant::now() < deadline, "file search did not finish");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn mock_index_ranks_exact_prefix_and_substring_matches() {
        let plugin = FileSearchPlugin::with_mock_index(&[
            r"C:\Users\Test\Documents\project-notes.txt",
            r"C:\Users\Test\Documents\project.txt",
            r"C:\Users\Test\Downloads\old-project.zip",
        ]);

        let results = await_results(&plugin, "project");
        let titles = results
            .iter()
            .map(|result| result.title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            titles,
            vec!["project.txt", "project-notes.txt", "old-project.zip"]
        );
    }

    #[test]
    fn matching_is_unicode_safe() {
        let plugin = FileSearchPlugin::with_mock_index(&[
            r"C:\Users\Test\Documents\Résumé Été.txt",
            r"C:\Users\Test\Documents\Resume.txt",
        ]);

        let results = await_results(&plugin, "résumé");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Résumé Été.txt");
    }

    #[test]
    fn results_are_deterministic_and_capped() {
        let paths = (0..12)
            .map(|index| format!(r"C:\Users\Test\Documents\report-{index:02}.txt"))
            .collect::<Vec<_>>();
        let entries = paths
            .iter()
            .filter_map(|path| FileEntry::from_path(PathBuf::from(path)))
            .collect::<Vec<_>>();

        let first = rank_entries(entries.iter(), "report");
        let second = rank_entries(entries.iter(), "report");

        assert_eq!(first.len(), MAX_RESULTS);
        assert_eq!(
            first.iter().map(|entry| &entry.path).collect::<Vec<_>>(),
            second.iter().map(|entry| &entry.path).collect::<Vec<_>>()
        );
    }

    #[test]
    fn dispatcher_prefix_is_textual_file_command() {
        let plugin = FileSearchPlugin::with_mock_index(&[]);

        assert_eq!(plugin.prefix(), Some("file"));
        assert!(!plugin.is_global());
    }

    #[test]
    fn query_returns_without_waiting_for_debounce_and_uses_latest_input() {
        let plugin = FileSearchPlugin::with_mock_index(&[
            r"C:\Users\Test\Documents\alpha.txt",
            r"C:\Users\Test\Documents\beta.txt",
        ]);
        let started = Instant::now();
        assert_eq!(plugin.query("alpha")[0].id, "file:indexing");
        assert!(started.elapsed() < SEARCH_DEBOUNCE);
        let results = await_results(&plugin, "beta");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "beta.txt");
    }
}
