use super::{config::Colors, math::selected_sector};
use gpui::*;
use std::f32::consts::PI;

const COLORS: Colors = Colors::DEFAULT;
const RING_RADIUS: f32 = 20.0;
const RING_THICKNESS: f32 = 8.0;
const ITEM_ORBIT_RADIUS: f32 = 160.0;

use gpui::SharedString;

#[derive(Clone, Default)]
pub struct MenuItem {
    pub label: SharedString,
    // pub function: Fn,
}

pub struct PieMenuView {
    pub x: f32,
    pub y: f32,
    pub items: Vec<MenuItem>,
    cursor_angle: f32,
}

impl PieMenuView {
    pub fn new(x: f32, y: f32, items: Vec<MenuItem>) -> Self {
        Self {
            x,
            y,
            items,
            cursor_angle: -PI / 2.0,
        }
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (mouse_x, mouse_y): (f32, f32) = (event.position.x.into(), event.position.y.into());
        let (dx, dy) = (mouse_x - self.x, mouse_y - self.y);

        if dx * dx + dy * dy > 4.0 {
            self.cursor_angle = dy.atan2(dx);
            cx.notify();
        }
    }

    fn render_ring(&self) -> impl IntoElement {
        div()
            .absolute()
            .left(px(self.x - RING_RADIUS))
            .top(px(self.y - RING_RADIUS))
            .size(px(RING_RADIUS * 2.0))
            .rounded_full()
            .border(px(RING_THICKNESS))
            .border_color(rgb(COLORS.surface))
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(COLORS.text))
    }

    fn render_highlight_arc(&self, count: usize) -> impl IntoElement {
        let mid_radius = RING_RADIUS - RING_THICKNESS * 0.5;
        let stroke_width = px(RING_THICKNESS);
        let sector_angle = 2.0 * PI / count.max(1) as f32;
        let (start_angle, end_angle) = (
            self.cursor_angle - sector_angle / 2.0,
            self.cursor_angle + sector_angle / 2.0,
        );
        let (cx, cy) = (self.x, self.y);

        canvas(
            |_, _, _| {},
            move |_, _, window, _| {
                let start = point(
                    px(cx + mid_radius * start_angle.cos()),
                    px(cy + mid_radius * start_angle.sin()),
                );
                let end = point(
                    px(cx + mid_radius * end_angle.cos()),
                    px(cy + mid_radius * end_angle.sin()),
                );

                let mut builder = PathBuilder::stroke(stroke_width);
                builder.move_to(start);
                builder.arc_to(
                    point(px(mid_radius), px(mid_radius)),
                    px(0.0),
                    sector_angle > PI,
                    true,
                    end,
                );

                if let Ok(path) = builder.build() {
                    window.paint_path(path, rgb(COLORS.arc));
                }
            },
        )
        .absolute()
        .size_full()
    }

    fn render_menu_items(&self, selected: usize) -> impl Iterator<Item = impl IntoElement> + '_ {
        let count = self.items.len() as f32;
        self.items.iter().enumerate().map(move |(i, item)| {
            let angle = -PI / 2.0 + (i as f32 / count) * 2.0 * PI;
            let (item_x, item_y) = (
                self.x + ITEM_ORBIT_RADIUS * angle.cos(),
                self.y + ITEM_ORBIT_RADIUS * angle.sin(),
            );

            let bg = rgb(if i == selected {
                COLORS.surface_hover
            } else {
                COLORS.surface
            });

            div()
                .absolute()
                .left(px(item_x))
                .top(px(item_y))
                .w(px(0.))
                .h(px(0.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1p5()
                        .bg(bg)
                        .rounded_xl()
                        .border_1()
                        .border_color(rgb(COLORS.surface))
                        .child(
                            div()
                                .flex()
                                .justify_center()
                                .text_color(rgb(COLORS.text))
                                .text_sm()
                                .child(item.label.clone()),
                        ),
                )
        })
    }
}

impl Render for PieMenuView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.items.len();
        let selected = selected_sector(self.cursor_angle, count);

        div()
            .size_full()
            .relative()
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .child(self.render_ring())
            .child(self.render_highlight_arc(count))
            .children(self.render_menu_items(selected))
    }
}
