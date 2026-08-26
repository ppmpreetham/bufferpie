use super::save;
use crate::ui::{config::AppConfig, pie_menu::PieMenu};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{InputEvent, Textarea, TextareaState};

pub struct ManualEditor {
    pub config: Entity<AppConfig>,
    pub editor: Entity<TextareaState>,
    pub error: Option<SharedString>,
    /// serialized menus waiting to be pushed into the editor (auto tab changed)
    pending: Option<String>,
    _subscriptions: Vec<Subscription>,
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
        let subscriptions = vec![
            cx.subscribe(&editor, |this, _, ev, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.try_apply(cx);
                }
            }),
            cx.observe(&config, |this, _, cx| this.queue_sync(cx)),
        ];
        Self {
            config,
            editor,
            error: None,
            pending: None,
            _subscriptions: subscriptions,
        }
    }

    /// schedules an editor refresh when the auto tab edited the menus
    fn queue_sync(&mut self, cx: &mut Context<Self>) {
        let json = serde_json::to_string_pretty(&self.config.read(cx).menus).unwrap_or_default();
        if self.editor.read(cx).value() != json && self.pending.is_none() {
            self.pending = Some(json);
            cx.notify();
        }
    }

    fn flush_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(json) = self.pending.take() {
            cx.spawn_in(window, async move |this, cx| {
                this.update_in(cx, |this, window, cx| {
                    this.editor
                        .update(cx, |state, cx| state.set_value(json, window, cx));
                })
            })
            .detach();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.flush_pending(window, cx);

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
