pub mod assets;
pub mod colors;
pub mod config;
pub mod math;
pub mod pie_menu;
pub mod settings;

use gpui::*;
use key::MenuState;
use pie_menu::PieMenuView;
use settings::load;
use std::sync::Arc;

use crate::key;

pub fn create_pie_menu_window(
    cx: &mut App,
    state: Arc<MenuState>,
) -> Result<WindowHandle<PieMenuView>> {
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
        |_, cx| {
            let config = cx.new(|_| load());
            cx.new(|_| PieMenuView::new(config, state))
        },
    )
}
