pub mod config;
pub mod math;
pub mod pie_menu;

use gpui::*;
use pie_menu::{MenuItem, PieMenu, PieMenuView};

fn default_menus() -> Vec<PieMenu> {
    vec![
        PieMenu {
            name: "Edit".into(),
            items: vec![
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
            ],
        },
        PieMenu {
            name: "Window".into(),
            items: vec![
                MenuItem {
                    label: "Minimize".into(),
                },
                MenuItem {
                    label: "Maximize".into(),
                },
                MenuItem {
                    label: "Close".into(),
                },
            ],
        },
    ]
}

pub fn create_pie_menu_window(cx: &mut App) -> Result<WindowHandle<PieMenuView>> {
    cx.text_system()
        .add_fonts(vec![include_bytes!("public/ReciaDisplay.ttf").into()])
        .expect("failed to load custom font");

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                point(px(0.0), px(0.0)),
                size(px(0.0), px(0.0)),
            ))),
            titlebar: None,
            focus: false,
            show: true,
            is_movable: false,
            is_resizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: Some(WindowDecorations::Client),
            kind: WindowKind::PopUp,
            ..Default::default()
        },
        |_, cx| cx.new(|_| PieMenuView::new(default_menus())),
    )
}
