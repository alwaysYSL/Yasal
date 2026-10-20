use crate::platform::win32;
use egui::{ColorImage, Context, TextureHandle, TextureId, TextureOptions};
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

struct LoadedIcon {
    identifier: String,
    image: Option<ColorImage>,
}

pub struct IconCache {
    cache_dir: PathBuf,
    textures: HashMap<String, TextureHandle>,
    pending: HashSet<String>,
    failed: HashSet<String>,
    sender: Sender<LoadedIcon>,
    receiver: Receiver<LoadedIcon>,
}

impl IconCache {
    pub fn new() -> Self {
        let local_data = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = local_data.join("Yasal").join("IconCache");
        let _ = fs::create_dir_all(&cache_dir);
        let (sender, receiver) = mpsc::channel();

        Self {
            cache_dir,
            textures: HashMap::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            sender,
            receiver,
        }
    }

    /// Computes a deterministic cache path that changes when the source file changes.
    pub fn get_icon_path(&self, identifier: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        identifier.hash(&mut hasher);
        if let Ok(modified) = fs::metadata(identifier).and_then(|metadata| metadata.modified()) {
            modified.hash(&mut hasher);
        }
        self.cache_dir.join(format!("{:016x}.png", hasher.finish()))
    }

    pub fn is_cached(&self, identifier: &str) -> bool {
        self.get_icon_path(identifier).exists()
    }

    /// Returns a ready GPU texture or starts a non-blocking shell extraction.
    pub fn texture_for(&mut self, ctx: &Context, identifier: &str) -> Option<TextureId> {
        self.collect_ready(ctx);

        if let Some(texture) = self.textures.get(identifier) {
            return Some(texture.id());
        }
        if self.pending.contains(identifier) || self.failed.contains(identifier) {
            return None;
        }

        self.pending.insert(identifier.to_owned());
        let identifier_owned = identifier.to_owned();
        let source_path = PathBuf::from(identifier);
        let cache_path = self.get_icon_path(identifier);
        let sender = self.sender.clone();
        let repaint_ctx = ctx.clone();

        std::thread::spawn(move || {
            let available = cache_path.exists()
                || win32::extract_and_cache_icon(Path::new(&source_path), &cache_path);
            let image = if available {
                load_color_image(&cache_path)
            } else {
                None
            };

            let _ = sender.send(LoadedIcon {
                identifier: identifier_owned,
                image,
            });
            repaint_ctx.request_repaint();
        });

        None
    }

    fn collect_ready(&mut self, ctx: &Context) {
        while let Ok(loaded) = self.receiver.try_recv() {
            self.pending.remove(&loaded.identifier);
            if let Some(image) = loaded.image {
                if self.textures.len() >= 64 {
                    self.textures.clear();
                }
                let texture = ctx.load_texture(
                    format!("app-icon:{}", loaded.identifier),
                    image,
                    TextureOptions::LINEAR,
                );
                self.textures.insert(loaded.identifier, texture);
            } else {
                self.failed.insert(loaded.identifier);
            }
        }
    }
}

fn load_color_image(path: &Path) -> Option<ColorImage> {
    let image = image::ImageReader::open(path)
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, image.as_raw()))
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
