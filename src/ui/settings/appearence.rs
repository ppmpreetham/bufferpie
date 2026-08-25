use gpui::*;
use gpui_component::Colorize;
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};

use super::save;
use crate::ui::config::AppConfig;

pub struct AppearanceTab {
    pub config: Entity<AppConfig>,
    arc: Entity<ColorPickerState>,
    surface: Entity<ColorPickerState>,
    surface_hover: Entity<ColorPickerState>,
    text: Entity<ColorPickerState>,
    _subscriptions: Vec<Subscription>,
}

impl AppearanceTab {
    pub fn new(config: Entity<AppConfig>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let colors = config.read(cx).colors.clone();
        let arc = cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(colors.arc)));
        let surface =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(colors.surface)));
        let surface_hover =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(colors.surface_hover)));
        let text = cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(colors.text)));

        let subs = vec![
            cx.subscribe(&arc, Self::on_color_change_arc),
            cx.subscribe(&surface, Self::on_color_change_surface),
            cx.subscribe(&surface_hover, Self::on_color_change_surface_hover),
            cx.subscribe(&text, Self::on_color_change_text),
        ];

        Self {
            config,
            arc,
            surface,
            surface_hover,
            text,
            _subscriptions: subs,
        }
    }

    fn on_color_change_arc(
        &mut self,
        _: Entity<ColorPickerState>,
        ev: &ColorPickerEvent,
        cx: &mut Context<Self>,
    ) {
        if let ColorPickerEvent::Change(Some(color)) = ev {
            self.config.update(cx, |c, cx| {
                c.colors.arc =
                    u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
                let _ = save(c);
                cx.notify();
            });
        }
    }

    fn on_color_change_surface(
        &mut self,
        _: Entity<ColorPickerState>,
        ev: &ColorPickerEvent,
        cx: &mut Context<Self>,
    ) {
        if let ColorPickerEvent::Change(Some(color)) = ev {
            self.config.update(cx, |c, cx| {
                c.colors.surface =
                    u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
                let _ = save(c);
                cx.notify();
            });
        }
    }

    fn on_color_change_surface_hover(
        &mut self,
        _: Entity<ColorPickerState>,
        ev: &ColorPickerEvent,
        cx: &mut Context<Self>,
    ) {
        if let ColorPickerEvent::Change(Some(color)) = ev {
            self.config.update(cx, |c, cx| {
                c.colors.surface_hover =
                    u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
                let _ = save(c);
                cx.notify();
            });
        }
    }

    fn on_color_change_text(
        &mut self,
        _: Entity<ColorPickerState>,
        ev: &ColorPickerEvent,
        cx: &mut Context<Self>,
    ) {
        if let ColorPickerEvent::Change(Some(color)) = ev {
            self.config.update(cx, |c, cx| {
                c.colors.text =
                    u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
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
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child("Arc")
                    .child(ColorPicker::new(&self.arc)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child("Surface")
                    .child(ColorPicker::new(&self.surface)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child("Surface Hover")
                    .child(ColorPicker::new(&self.surface_hover)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child("Text")
                    .child(ColorPicker::new(&self.text)),
            )
    }
}
