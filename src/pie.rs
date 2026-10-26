use crate::config::{self, AppConfig};
use crate::key::{self, MenuAction};
use crate::{assets::node_icon, platform, settings};
use gpui_kit::*;
use std::f32::consts::{PI, TAU};
use std::time::Duration;
use zopra::prelude::*;

const RING_RADIUS: f32 = 20.0;
const RING_THICKNESS: f32 = 8.0;
const ITEM_ORBIT_RADIUS: f32 = 160.0;

/// what the overlay shows, driven by the global input monitor
#[derive(Clone, Copy)]
struct Pie {
    x: f32,
    y: f32,
    menu: usize,
    /// cursor angle around the center
    angle: f32,
    visible: bool,
    settings_visible: bool,
    settings_hovered: bool,
}

impl Pie {
    fn close(&mut self) {
        self.visible = false;
        self.settings_hovered = false;
    }
}

/// root of the transparent, full screen overlay window
pub struct Overlay {
    pie: Signal<Pie>,
    config: Entity<AppConfig>,
}

impl Render for Overlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        view! { <PieMenu pie={self.pie} config={self.config.clone()} /> }
    }
}

impl Overlay {
    pub fn handle(&mut self, action: MenuAction, window: &mut Window, cx: &mut Context<Self>) {
        let pie = self.pie;
        match action {
            MenuAction::Open { x, y } => {
                pie.update(|pie| *pie = Pie { x, y, visible: true, settings_hovered: false, ..*pie }, cx);
                platform::raise(window);
            }
            // releasing caps lock fires the hovered item (or settings) before closing
            MenuAction::Close => {
                let state = pie.get(cx);
                if state.visible && state.settings_hovered {
                    open_settings(&self.config, cx);
                } else if state.visible {
                    run_item(&self.config, state.menu, |count| selected_sector(state.angle, count), cx);
                }
                pie.update(Pie::close, cx);
            }
            MenuAction::Cancel => pie.update(Pie::close, cx),
            MenuAction::ShowSettingsButton => pie.update(|pie| pie.settings_visible = true, cx),
            MenuAction::HideSettingsButton => pie.update(|pie| (pie.settings_visible, pie.settings_hovered) = (false, false), cx),
            // recorded keys changed, repaint the macro form
            MenuAction::KeysChanged => settings::refresh(cx),
            MenuAction::OpenSettings => settings::open(self.config.clone(), cx),
        }
        // a 1px window keeps the overlay out of the way while idle
        let state = pie.get(cx);
        let active = state.visible || state.settings_visible;
        window.resize(if active { Bounds::maximized(None, cx).size } else { size(px(1.0), px(1.0)) });
    }
}

pub fn open(cx: &mut App) -> Result<WindowHandle<Overlay>> {
    cx.text_system()
        .add_fonts(vec![include_bytes!("../assets/ReciaDisplay.ttf").into()])
        .expect("failed to load custom font");

    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(px(0.0), px(0.0)), size(px(1.0), px(1.0))))),
        titlebar: None,
        focus: false,
        show: true,
        is_movable: false,
        is_resizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        window_decorations: Some(WindowDecorations::Client),
        kind: WindowKind::PopUp,
        ..Default::default()
    };
    let window = cx.open_window(options, |_, cx| {
        let config = cx.new(|_| config::load());
        cx.new(|cx| {
            let pie = Pie { x: 0.0, y: 0.0, menu: 0, angle: -PI / 2.0, visible: false, settings_visible: false, settings_hovered: false };
            Overlay { pie: Signal::new(pie, Some(cx.entity_id()), cx), config }
        })
    })?;
    _ = window.update(cx, |_, window, _| platform::make_no_activate(window));
    Ok(window)
}

/// the item of `menu` that `pick` chooses out of its item count, run off the ui thread
fn run_item(config: &Entity<AppConfig>, menu: usize, pick: impl FnOnce(usize) -> usize, cx: &App) {
    let items = config.read(cx).menus.get(menu).map_or(&[][..], |menu| &menu.items);
    if let Some(action) = items.get(pick(items.len())).and_then(|item| item.action.clone()) {
        // keep the ui thread free while macros strike keys
        std::thread::spawn(move || action.run());
    }
}

/// opening settings dismisses the pie like esc would
fn open_settings(config: &Entity<AppConfig>, cx: &mut App) {
    key::deactivate();
    settings::open(config.clone(), cx);
}

fn selected_sector(angle: f32, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let sector = TAU / count as f32;
    ((angle + PI / 2.0 + sector / 2.0).rem_euclid(TAU) / sector) as usize % count
}

#[component]
fn pie_menu(pie: Signal<Pie>, config: Entity<AppConfig>) {
    let AppConfig { menus, colors, .. } = config.read(cx).clone();
    let (text, menu_count) = (rgb(colors.text), menus.len());
    let fill = |hovered: bool| rgb(if hovered { colors.surface_hover } else { colors.surface });
    let state = pie();

    let settings = view! {
        <div id="settings-btn" class="absolute bottom-4 right-4 size-12 rounded-full flex items-center justify-center"
            bg={fill(state.settings_hovered)}
            text_color={text}
            on_mouse_down={(MouseButton::Left, {
                let config = config.clone();
                move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| {
                    open_settings(&config, cx);
                    pie.update(Pie::close, cx);
                }
            })}
            on_hover={|hovered: &bool| pie(|pie| pie.settings_hovered = *hovered)}
        >
            <svg src="logos/settings.svg" class="size-5" text_color={text} />
        </div>
    };

    let menu_ix = state.menu.min(menu_count.saturating_sub(1));
    pie.update_untracked(|pie| pie.menu = menu_ix);
    let Some(menu) = menus.get(menu_ix).filter(|_| state.visible) else {
        return view! { <div class="size-full">{...state.settings_visible.then_some(settings)}</div> };
    };
    let count = menu.items.len();
    let selected = if state.settings_hovered { usize::MAX } else { selected_sector(state.angle, count) };

    view! {
        <div class="size-full relative"
            on_mouse_move={|event: &MouseMoveEvent| pie(|pie| {
                let (x, y): (f32, f32) = (event.position.x.into(), event.position.y.into());
                pie.angle = (y - pie.y).atan2(x - pie.x).rem_euclid(TAU);
            })}
            on_scroll_wheel={|event: &ScrollWheelEvent| {
                let dy: f32 = event.delta.pixel_delta(px(20.0)).y.into();
                if menu_count > 0 && dy != 0.0 {
                    let step = if dy > 0.0 { 1 } else { menu_count - 1 };
                    pie(|pie| pie.menu = (pie.menu + step) % menu_count);
                }
            }}
            // clicking anywhere outside the items dismisses the menu
            on_mouse_down={(MouseButton::Left, move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| pie.update(Pie::close, cx))}
        >
            <div class="absolute rounded-full border-8"
                left={px(state.x - RING_RADIUS)}
                top={px(state.y - RING_RADIUS)}
                size={px(RING_RADIUS * 2.0)}
                border_color={rgb(colors.surface)}
            />
            // the arc spins uselessly on a single item, only draw it for 2+
            {...(count > 1 && !state.settings_hovered).then(|| highlight_arc(state, count, colors.arc))}
            {...menu.items.iter().enumerate().map(|(i, item)| {
                let angle = -PI / 2.0 + i as f32 / count as f32 * TAU;
                let (dx, dy) = (ITEM_ORBIT_RADIUS * angle.cos(), ITEM_ORBIT_RADIUS * angle.sin());
                let config = config.clone();
                view! {
                    <div class="absolute size-0 flex items-center justify-center"
                        on_mouse_down={(MouseButton::Left, move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| {
                            run_item(&config, menu_ix, |_| i, cx);
                            pie.update(Pie::close, cx);
                        })}
                    >
                        <div class="flex flex-row items-center justify-center gap-2 px-3 py-1.5 rounded-md border w-32"
                            bg={fill(i == selected)}
                            border_color={rgb(colors.surface)}
                            text_color={text}
                        >
                            {node_icon(item.action.as_ref(), text)}
                            {item.label.clone()}
                        </div>
                    </div>
                }
                .with_animation(
                    ("pie-item", i),
                    Animation::new(Duration::from_millis(100)).with_easing(ease_out_quint()),
                    move |this, delta| this.left(px(state.x + delta * dx)).top(px(state.y + delta * dy)),
                )
            })}
            <div class="absolute top-6 right-8" font_family="ReciaDisplay" text_size={px(72.0)} text_color={text}>
                {menu.name.clone()}
            </div>
            {...state.settings_visible.then_some(settings)}
        </div>
    }
}

/// the ring segment pointing at the hovered item
fn highlight_arc(state: Pie, count: usize, color: u32) -> impl IntoElement {
    let sector = TAU / count as f32;
    let radius = px(RING_RADIUS - RING_THICKNESS * 0.5);
    let at = move |angle: f32| point(px(state.x) + radius * angle.cos(), px(state.y) + radius * angle.sin());
    view! {
        <canvas class="absolute size-full"
            prepaint={|_, _, _| {}}
            paint={move |_, _, window: &mut Window, _| {
                let mut path = PathBuilder::stroke(px(RING_THICKNESS));
                path.move_to(at(state.angle - sector / 2.0));
                path.arc_to(point(radius, radius), px(0.0), sector > PI, true, at(state.angle + sector / 2.0));
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(color));
                }
            }}
        />
    }
}
