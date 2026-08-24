use gpui::*;

pub struct PieMenuView {
    pub x: i32,
    pub y: i32,
}
const CIRCLE_SIZE: f32 = 100.0;

impl Render for PieMenuView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().relative().child(
            div()
                .absolute()
                .left(px(self.x as f32 - CIRCLE_SIZE / 2.0))
                .top(px(self.y as f32 - CIRCLE_SIZE / 2.0))
                .size(px(CIRCLE_SIZE))
                .bg(rgb(0x1e1e2e))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0xcdd6f4))
                .child("Pie Menu"),
        )
    }
}

pub fn open_pie_menu(cx: &mut App, x: i32, y: i32) -> Result<WindowHandle<PieMenuView>> {
    let bounds = Bounds::maximized(None, cx);

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
        |_, cx| cx.new(|_| PieMenuView { x, y }),
    )
}
