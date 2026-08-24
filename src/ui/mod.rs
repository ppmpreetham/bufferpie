pub mod config;
pub mod math;
pub mod pie_menu;

use gpui::*;
use pie_menu::{MenuItem, PieMenuView};

pub fn open_pie_menu(cx: &mut App, x: f32, y: f32) -> Result<WindowHandle<PieMenuView>> {
    let bounds = Bounds::maximized(None, cx);

    let items = vec![
        MenuItem {
            label: "Copy".into(),
        },
        MenuItem {
            label: "Paste".into(),
        },
        MenuItem {
            label: "Cut".into(),
        },
        MenuItem {
            label: "Undo".into(),
        },
    ];

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            is_resizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: Some(WindowDecorations::Client),
            kind: WindowKind::PopUp,
            ..Default::default()
        },
        move |_, cx| cx.new(|_| PieMenuView::new(x, y, items)),
    )
}
