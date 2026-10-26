use crate::actions::Action;
use gpui_kit::{AnyElement, AssetSource, Hsla, IntoElement, Result, SharedString, Styled};
use rust_embed::RustEmbed;
use std::{borrow::Cow, path::{Path, PathBuf}, sync::Mutex};
use zopra::{utils::key_hash, view};

#[derive(RustEmbed)]
#[folder = "assets"]
struct Embedded;

/// serves the embedded logos and extracted app icons, falling back to gpui-kit's assets
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.starts_with("app-icons/") {
            return Ok(std::fs::read(cache_dir().join(path)).ok().map(Cow::Owned));
        }
        Embedded::get(path).map_or_else(|| gpui_kit::assets::Assets.load(path), |file| Ok(Some(file.data)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}

/// logo shown left of an item's label: the app's own icon once extracted, else a glyph
pub fn node_icon(action: Option<&Action>, color: impl Into<Hsla>) -> AnyElement {
    let glyph = match action {
        Some(Action::App { path }) => {
            let file = format!("app-icons/{:x}.png", key_hash(path));
            if cache_dir().join(&file).exists() {
                return view! { <img src={file} class="size-4" /> }.into_any_element();
            }
            extract_icon(path);
            "logos/command.svg"
        }
        Some(Action::Macro { .. }) => "logos/keyboard.svg",
        _ => "logos/command.svg",
    };
    view! { <svg src={glyph} class="size-4" text_color={color.into()} /> }.into_any_element()
}

/// pulls the icon out of an executable into the cache, once per executable
pub fn extract_icon(exe: &Path) {
    static PENDING: Mutex<Vec<u64>> = Mutex::new(Vec::new());
    let hash = key_hash(exe);
    {
        let mut pending = PENDING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if pending.contains(&hash) {
            return;
        }
        pending.push(hash);
    }
    let dir = cache_dir().join("app-icons");
    _ = std::fs::create_dir_all(&dir);
    _ = app_icon::extract_icon(exe, &dir.join(format!("{hash:x}.png")));
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("pinews")
}
