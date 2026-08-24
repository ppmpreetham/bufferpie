use gpui::*;

pub struct PieMenuView {
    pub center_x: i32,
    pub center_y: i32,
}

impl Render for PieMenuView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .left(px(self.center_x as f32 - 100.0))
            .top(px(self.center_y as f32 - 100.0))
            .size(px(200.0))
            .bg(rgb(0x1e1e2e))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0xcdd6f4))
            .child("Pie Menu Active")
    }
}

pub fn open_pie_menu(cx: &mut App, x: i32, y: i32) -> Result<WindowHandle<PieMenuView>> {
    let bounds = Bounds::centered(None, size(px(400.0), px(400.0)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            ..Default::default()
        },
        |_, cx| {
            cx.new(|_| PieMenuView {
                center_x: x,
                center_y: y,
            })
        },
    )
}
