use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
};

use super::save;
use crate::ui::colors::Colors;
use crate::ui::config::AppConfig;

/// which color of the theme a picker edits
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Arc,
    Surface,
    SurfaceHover,
    Text,
}

impl Slot {
    const ALL: [Self; 4] = [Self::Arc, Self::Surface, Self::SurfaceHover, Self::Text];

    fn label(self) -> &'static str {
        match self {
            Self::Arc => "Arc",
            Self::Surface => "Surface",
            Self::SurfaceHover => "Surface Hover",
            Self::Text => "Text",
        }
    }

    fn get(self, colors: &Colors) -> u32 {
        match self {
            Self::Arc => colors.arc,
            Self::Surface => colors.surface,
            Self::SurfaceHover => colors.surface_hover,
            Self::Text => colors.text,
        }
    }

    fn set(self, colors: &mut Colors, value: u32) {
        match self {
            Self::Arc => colors.arc = value,
            Self::Surface => colors.surface = value,
            Self::SurfaceHover => colors.surface_hover = value,
            Self::Text => colors.text = value,
        }
    }
}

pub struct AppearanceTab {
    pub config: Entity<AppConfig>,
    pickers: Vec<(Slot, Entity<ColorPickerState>)>,
    _subscriptions: Vec<Subscription>,
}

impl AppearanceTab {
    pub fn new(config: Entity<AppConfig>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let colors = config.read(cx).colors.clone();
        let pickers: Vec<_> = Slot::ALL
            .map(|slot| {
                (
                    slot,
                    cx.new(|cx| {
                        ColorPickerState::new(window, cx).default_value(rgb(slot.get(&colors)))
                    }),
                )
            })
            .into();
        let subscriptions = pickers
            .iter()
            .map(|(slot, state)| {
                let slot = *slot;
                cx.subscribe(state, move |this, _, ev, cx| this.on_change(slot, ev, cx))
            })
            .collect();

        Self {
            config,
            pickers,
            _subscriptions: subscriptions,
        }
    }

    fn on_change(&mut self, slot: Slot, ev: &ColorPickerEvent, cx: &mut Context<Self>) {
        if let ColorPickerEvent::Change(Some(color)) = ev {
            let value =
                u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
            self.config.update(cx, |c, cx| {
                slot.set(&mut c.colors, value);
                let _ = save(c);
                cx.notify();
            });
        }
    }
}

impl Render for AppearanceTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .children(self.pickers.iter().map(|(slot, state)| {
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child(slot.label())
                    .child(ColorPicker::new(state))
            }))
    }
}
