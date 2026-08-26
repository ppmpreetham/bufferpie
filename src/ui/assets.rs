use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

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
