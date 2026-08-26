use super::appearance::AppearanceTab;
use super::config_tab::ConfigTab;
use super::manual::ManualEditor;
use super::menus::MenusEditor;
use super::viewer::SettingsView;
use crate::ui::config::AppConfig;
use gpui::*;

pub fn open_settings_window(config: Entity<AppConfig>, cx: &mut App) {
    if let Some(existing) = cx
        .windows()
        .into_iter()
        .find_map(|w| w.downcast::<gpui_component::Root>())
    {
        let _ = existing.update(cx, |_, window, _| window.activate_window());
        return;
    }

    let config_entity = config.clone();
    let _ = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(720.0), px(480.0)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("Settings".into()),
                ..Default::default()
            }),
            kind: WindowKind::Normal,
            is_resizable: true,
            ..Default::default()
        },
        |window, cx| {
            let auto = cx.new(|_| MenusEditor::new(config_entity.clone()));
            let manual = cx.new(|cx| ManualEditor::new(window, cx, config_entity.clone()));

            let mode = config_entity.read(cx).mode.clone();
            let config_tab = cx.new(|_| ConfigTab {
                mode,
                auto,
                manual,
                config: config_entity.clone(),
            });

            let appearance_tab = cx.new(|cx| AppearanceTab::new(config_entity.clone(), window, cx));
            let view = cx.new(|_| SettingsView::new(appearance_tab, config_tab));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        },
    );
}
