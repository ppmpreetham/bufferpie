use crate::actions::types::Action;
use gpui::{AssetSource, Result, SharedString};
use parking_lot::Mutex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// serves the embedded logo svgs, falling back to gpui component assets
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match path {
            "logos/command.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/command.svg"
            )))),
            "logos/keyboard.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/keyboard.svg"
            )))),
            "logos/settings.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/settings.svg"
            )))),
            "logos/appearence.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/appearence.svg"
            )))),
            "logos/config.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/config.svg"
            )))),
            "logos/cancel.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "public/logos/cancel.svg"
            )))),
            _ => gpui_component_assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_component_assets::Assets.list(path)
    }
}

/// the logo to show for a node, either an embedded svg or an extracted app icon
pub enum NodeIcon {
    Svg(&'static str),
    /// relative asset path of an extracted exe icon
    File(String),
}

/// logo shown left of an item's label
pub fn node_icon(action: Option<&Action>) -> NodeIcon {
    match action {
        Some(Action::App { path }) => {
            let file = format!("app-icons/{:x}.png", hash_path(path));
            if icon_file(&file).exists() {
                NodeIcon::File(file)
            } else {
                // not extracted yet, fall back to the generic glyph
                extract_icon(path);
                NodeIcon::Svg("logos/command.svg")
            }
        }
        Some(Action::Macro { .. }) => NodeIcon::Svg("logos/keyboard.svg"),
        Some(Action::Command { .. }) | None => NodeIcon::Svg("logos/command.svg"),
    }
}

/// executables already queued for extraction
fn pending() -> &'static Mutex<HashSet<u64>> {
    static PENDING: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();
    PENDING.get_or_init(Mutex::default)
}

fn icon_file(asset: &str) -> PathBuf {
    cache_dir().join(asset)
}

/// pulls the associated icon out of an exe with a headless powershell one-liner,
/// deduplicated per executable
pub fn extract_icon(exe: &Path) {
    let hash = hash_path(exe);
    if !pending().lock().insert(hash) {
        return;
    }

    let out = icon_file(&format!("app-icons/{hash:x}.png"));
    let Some(exe) = exe.to_str() else { return };
    let Some(out) = out.to_str() else { return };
    let _ = std::fs::create_dir_all(cache_dir().join("app-icons"));

    let script = format!(
        "Add-Type -AssemblyName System.Drawing;\
         [System.Drawing.Icon]::ExtractAssociatedIcon('{exe}')\
         .ToBitmap().Save('{out}',[System.Drawing.Imaging.ImageFormat]::Png)"
    );

    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd.status();
}

fn hash_path(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("pinews")
}
