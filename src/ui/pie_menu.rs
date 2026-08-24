use gpui::*;
use std::f32::consts::PI;
use std::time::Duration;

use crate::actions::types::Action;

use super::config::Colors;
use super::math::{normalize_angle, selected_sector};

const RING_RADIUS: f32 = 20.0;
const RING_THICKNESS: f32 = 8.0;
const ITEM_ORBIT_RADIUS: f32 = 160.0;
const SIDE_SLOT_WIDTH: f32 = 32.0;

#[derive(Clone)]
pub struct Item {
    pub label: SharedString,
    // pub action: Option<Action>,
}

pub struct PieMenu {
    pub name: SharedString,
    pub items: Vec<Item>,
}

pub struct PieMenuView {
    pub x: f32,
    pub y: f32,
    pub menus: Vec<PieMenu>,
    pub current_menu: usize,
    pub visible: bool,
    cursor_angle: f32,
}

impl PieMenuView {
    pub fn new(menus: Vec<PieMenu>) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            menus,
            current_menu: 0,
            visible: false,
            cursor_angle: -PI / 2.0,
        }
    }

    fn handle_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menus.is_empty() {
            return;
        }
        let dy: f32 = event.delta.pixel_delta(px(20.0)).y.into();
        if dy > 0.0 {
            self.current_menu = (self.current_menu + 1) % self.menus.len();
        } else if dy < 0.0 {
            self.current_menu = (self.current_menu + self.menus.len() - 1) % self.menus.len();
        }
        cx.notify();
    }

    pub fn open_at(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        self.x = x;
        self.y = y;
        self.visible = true;
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mouse_x: f32 = event.position.x.into();
        let mouse_y: f32 = event.position.y.into();
        let dx = mouse_x - self.x;
        let dy = mouse_y - self.y;
        self.cursor_angle = normalize_angle(dy.atan2(dx));
        cx.notify();
    }
}

impl Render for PieMenuView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().size(px(0.0));
        }

        let colors = Colors::DEFAULT;
        let center_x = self.x;
        let center_y = self.y;
        let menu = &self.menus[self.current_menu];
        let selected = selected_sector(self.cursor_angle, menu.items.len());
        let menu_name = menu.name.clone();
        let items = menu.items.clone();

        div()
            .size_full()
            .relative()
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_scroll_wheel(cx.listener(Self::handle_scroll))
            .child(render_ring(center_x, center_y, &colors))
            .child(render_highlight_arc(
                center_x,
                center_y,
                self.cursor_angle,
                items.len(),
                &colors,
            ))
            .children(render_menu_items(
                center_x, center_y, &items, selected, &colors,
            ))
            .child(
                div()
                    .absolute()
                    .top(px(24.0))
                    .right(px(32.0))
                    .font_family("ReciaDisplay")
                    .text_size(px(72.0))
                    .text_color(rgb(colors.text))
                    .child(menu_name),
            )
    }
}

fn render_ring(center_x: f32, center_y: f32, colors: &Colors) -> impl IntoElement {
    div()
        .absolute()
        .left(px(center_x - RING_RADIUS))
        .top(px(center_y - RING_RADIUS))
        .size(px(RING_RADIUS * 2.0))
        .rounded_full()
        .border(px(RING_THICKNESS))
        .border_color(rgb(colors.surface))
}

fn render_highlight_arc(
    center_x: f32,
    center_y: f32,
    angle: f32,
    item_count: usize,

    colors: &Colors,
) -> impl IntoElement {
    let arc_color = colors.arc;

    canvas(
        |_, _, _| {},
        move |_, _, window, _cx| {
            let radius = px(RING_RADIUS - RING_THICKNESS * 0.5);
            let sector_angle = 2.0 * PI / item_count.max(1) as f32;
            let half_angle = sector_angle * 0.5;

            let start_angle = angle - half_angle;
            let end_angle = angle + half_angle;
            let large_arc = sector_angle > PI;

            let start = point(
                px(center_x) + radius * start_angle.cos(),
                px(center_y) + radius * start_angle.sin(),
            );
            let end = point(
                px(center_x) + radius * end_angle.cos(),
                px(center_y) + radius * end_angle.sin(),
            );

            let mut builder = PathBuilder::stroke(px(RING_THICKNESS));
            builder.move_to(start);
            builder.arc_to(point(radius, radius), px(0.0), large_arc, true, end);

            if let Ok(path) = builder.build() {
                window.paint_path(path, rgb(arc_color));
            }
        },
    )
    .absolute()
    .size_full()
}

fn render_menu_items(
    center_x: f32,
    center_y: f32,
    items: &[Item],
    selected: usize,
    colors: &Colors,
) -> Vec<impl IntoElement> {
    if items.is_empty() {
        return Vec::new();
    }

    let count = items.len() as f32;

    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let angle = -PI / 2.0 + (i as f32 / count) * 2.0 * PI;
            let item_x = center_x + ITEM_ORBIT_RADIUS * angle.cos();
            let item_y = center_y + ITEM_ORBIT_RADIUS * angle.sin();
            let is_selected = i == selected;
            let item_id = ElementId::from(("pie-item", i));

            div()
                .absolute()
                .w(px(0.0))
                .h(px(0.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(rgb(if is_selected {
                            colors.surface_hover
                        } else {
                            colors.surface
                        }))
                        .w(px(SIDE_SLOT_WIDTH * 4.0))
                        .justify_center()
                        .text_color(rgb(colors.text))
                        .child(item.label.clone()),
                )
                .with_animation(
                    item_id,
                    Animation::new(Duration::from_millis(100)).with_easing(ease_out_quint()),
                    move |this, delta| {
                        let x = center_x + delta * (item_x - center_x);
                        let y = center_y + delta * (item_y - center_y);
                        this.left(px(x)).top(px(y))
                    },
                )
        })
        .collect()
}
