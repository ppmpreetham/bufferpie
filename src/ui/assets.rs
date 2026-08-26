use crate::actions::types::Action;
use gpui::{AssetSource, Result, SharedString};
use parking_lot::Mutex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// serves the embedded logo svgs and cached app icons,
/// falling back to gpui component assets
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
            p if p.starts_with("app-icons/") => {
                Ok(std::fs::read(cache_dir().join(p)).map(Cow::Owned).ok())
            }
            _ => gpui_component_assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_component_assets::Assets.list(path)
    }
}

/// logo asset shown left of an item's label
pub fn icon_for(action: Option<&Action>) -> String {
    match action {
        Some(Action::App { path }) => {
            let hash = hash_path(path);
            request_icon(hash, path.clone());
            format!("app-icons/{hash:x}.png")
        }
        Some(Action::Macro { .. }) => "logos/keyboard.svg".into(),
        Some(Action::Command { .. }) | None => "logos/command.svg".into(),
    }
}

/// executables waiting for their icon to be extracted
fn pending() -> &'static Mutex<HashMap<u64, PathBuf>> {
    static PENDING: OnceLock<Mutex<HashMap<u64, PathBuf>>> = OnceLock::new();
    PENDING.get_or_init(Mutex::default)
}

/// schedules one background icon extraction per executable
fn request_icon(hash: u64, exe: PathBuf) {
    let mut queue = pending().lock();
    if queue.contains_key(&hash) {
        return;
    }
    queue.insert(hash, exe.clone());

    let out = cache_dir().join(format!("app-icons/{hash:x}.png"));
    std::thread::spawn(move || extract_icon(&exe, &out));
}

/// pulls the associated icon out of an exe with a headless powershell one-liner
fn extract_icon(exe: &Path, out: &Path) {
    let (Some(exe), Some(out_str)) = (exe.to_str(), out.to_str()) else {
        return;
    };
    let _ = std::fs::create_dir_all(out.parent().unwrap_or(Path::new(".")));
    let script = format!(
        "Add-Type -AssemblyName System.Drawing;\
         [System.Drawing.Icon]::ExtractAssociatedIcon('{exe}')\
         .ToBitmap().Save('{out_str}',[System.Drawing.Imaging.ImageFormat]::Png)"
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
