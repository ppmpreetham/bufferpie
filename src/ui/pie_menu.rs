use super::assets::{NodeIcon, node_icon};
use super::colors::Colors;
use super::config::AppConfig;
use super::math::{normalize_angle, selected_sector};
use super::settings::window::open_settings_window;
use crate::actions::types::{Action, CellType, execute};
use crate::key::MenuState;
use gpui::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;

const RING_RADIUS: f32 = 20.0;
const RING_THICKNESS: f32 = 8.0;
const ITEM_ORBIT_RADIUS: f32 = 160.0;
const SIDE_SLOT_WIDTH: f32 = 32.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub label: SharedString,
    pub action: Option<Action>,
    pub celltype: CellType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PieMenu {
    pub name: SharedString,
    pub items: Vec<Item>,
}

pub struct PieMenuView {
    pub x: f32,
    pub y: f32,
    pub current_menu: usize,
    pub visible: bool,
    pub settings_visible: bool,
    cursor_angle: f32,
    pub config: Entity<AppConfig>,
    state: Arc<MenuState>,
}

impl PieMenuView {
    pub fn new(config: Entity<AppConfig>, state: Arc<MenuState>) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            current_menu: 0,
            visible: false,
            settings_visible: false,
            cursor_angle: -PI / 2.0,
            config,
            state,
        }
    }

    /// runs the hovered item's action and closes, used on caps lock release
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        if self.visible
            && let Some(action) = self
                .config
                .read(cx)
                .menus
                .get(self.current_menu)
                .and_then(|menu| {
                    menu.items
                        .get(selected_sector(self.cursor_angle, menu.items.len()))
                })
                .and_then(|item| item.action.clone())
        {
            // keep the ui thread free while macros strike keys
            std::thread::spawn(move || execute(&action));
        }
        self.close(cx);
    }

    /// runs a clicked item's action and closes
    fn pick(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.visible
            && let Some(action) = self
                .config
                .read(cx)
                .menus
                .get(self.current_menu)
                .and_then(|menu| menu.items.get(ix))
                .and_then(|item| item.action.clone())
        {
            std::thread::spawn(move || execute(&action));
        }
        self.close(cx);
    }

    fn handle_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let count = self.config.read(cx).menus.len();
        if count == 0 {
            return;
        }
        let dy: f32 = event.delta.pixel_delta(px(20.0)).y.into();
        if dy > 0.0 {
            self.current_menu = (self.current_menu + 1) % count;
        } else if dy < 0.0 {
            self.current_menu = (self.current_menu + count - 1) % count;
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

    pub fn show_settings_button(&mut self, cx: &mut Context<Self>) {
        self.settings_visible = true;
        cx.notify();
    }

    pub fn hide_settings_button(&mut self, cx: &mut Context<Self>) {
        self.settings_visible = false;
        cx.notify();
    }
}

impl Render for PieMenuView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.config.read(cx).colors.clone();

        let settings_btn = if self.settings_visible {
            div()
                .id("settings-btn")
                .absolute()
                .bottom(px(16.0))
                .right(px(16.0))
                .size(px(48.0))
                .rounded_full()
                .bg(rgb(colors.surface))
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(colors.text))
                .child(
                    svg()
                        .path("logos/settings.svg")
                        .size(px(20.0))
                        .text_color(rgb(colors.text)),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        // opening settings dismisses the pie like esc would
                        this.state.deactivate();
                        open_settings_window(this.config.clone(), cx);
                        this.close(cx);
                    }),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        if !self.visible {
            return div().size_full().child(settings_btn).into_any_element();
        }

        let center_x = self.x;
        let center_y = self.y;
        self.current_menu = self
            .current_menu
            .min(self.config.read(cx).menus.len().saturating_sub(1));
        let menu = &self.config.read(cx).menus[self.current_menu];
        let selected = selected_sector(self.cursor_angle, menu.items.len());
        let menu_name = menu.name.clone();
        let items = menu.items.clone();

        div()
            .size_full()
            .relative()
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_scroll_wheel(cx.listener(Self::handle_scroll))
            .child(render_ring(center_x, center_y, &colors))
            // the arc spins uselessly on a single item, only draw it for 2+
            .children((items.len() > 1).then(|| {
                render_highlight_arc(center_x, center_y, self.cursor_angle, items.len(), &colors)
            }))
            .children(render_menu_items(
                center_x,
                center_y,
                &items,
                selected,
                &colors,
                &cx.entity(),
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
            .child(settings_btn)
            .into_any_element()
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
    view: &Entity<PieMenuView>,
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
            let pick_view = view.clone();

            div()
                .absolute()
                .w(px(0.0))
                .h(px(0.0))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    pick_view.update(cx, |view, cx| view.pick(i, cx))
                })
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(rgb(if is_selected {
                            colors.surface_hover
                        } else {
                            colors.surface
                        }))
                        .border_1()
                        .border_color(rgb(colors.surface))
                        .w(px(SIDE_SLOT_WIDTH * 4.0))
                        .justify_center()
                        .text_color(rgb(colors.text))
                        .children([match node_icon(item.action.as_ref()) {
                            NodeIcon::Svg(path) => svg()
                                .path(path)
                                .size(px(16.0))
                                .text_color(rgb(colors.text))
                                .into_any_element(),
                            NodeIcon::File(path) => img(path).size(px(16.0)).into_any_element(),
                        }])
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
