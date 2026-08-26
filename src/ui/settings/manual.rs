use super::save;
use crate::ui::{config::AppConfig, pie_menu::PieMenu};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{InputEvent, Textarea, TextareaState};

pub struct ManualEditor {
    pub config: Entity<AppConfig>,
    pub editor: Entity<TextareaState>,
    pub error: Option<SharedString>,
    _subscription: Subscription,
}

impl ManualEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, config: Entity<AppConfig>) -> Self {
        let initial = serde_json::to_string_pretty(&config.read(cx).menus).unwrap_or_default();
        let editor = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(24)
                .placeholder("[{ \"name\": \"MENU\", \"items\": [] }]")
                .default_value(&initial)
        });
        let sub = cx.subscribe(&editor, |this, _, ev, cx| {
            if matches!(ev, InputEvent::Change) {
                this.try_apply(cx);
            }
        });
        Self {
            config,
            editor,
            error: None,
            _subscription: sub,
        }
    }

    fn try_apply(&mut self, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).value().to_string();
        match serde_json::from_str::<Vec<PieMenu>>(&text) {
            Ok(menus) => {
                self.error = None;
                self.config.update(cx, |c, cx| {
                    c.menus = menus;
                    let _ = save(c);
                    cx.notify();
                });
            }
            Err(err) => self.error = Some(err.to_string().into()),
        }
        cx.notify();
    }
}

impl Render for ManualEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .size_full()
            .child(Textarea::new(&self.editor).h_full())
            .when_some(self.error.clone(), |d, err| {
                d.child(div().text_color(rgb(0xf38ba8)).child(err))
            })
    }
}
