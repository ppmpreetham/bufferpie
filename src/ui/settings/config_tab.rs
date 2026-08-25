use crate::ui::config::AppConfig;

use super::ConfigMode;
use super::auto::AutoEditor;
use super::manual::ManualEditor;
use super::save;
use gpui::*;
use gpui_component::tab::{Tab, TabBar};

pub struct ConfigTab {
    pub mode: ConfigMode,
    pub auto: Entity<AutoEditor>,
    pub manual: Entity<ManualEditor>,
    pub config: Entity<AppConfig>,
}

impl Render for ConfigTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_auto = matches!(self.mode, ConfigMode::Auto);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div().p_2().child(
                    TabBar::new("config-mode")
                        .segmented()
                        .selected_index(if is_auto { 0 } else { 1 })
                        .child(Tab::new().label("Auto"))
                        .child(Tab::new().label("Manual"))
                        .on_click(cx.listener(|this, index: &usize, _, cx| {
                            this.mode = if *index == 0 {
                                ConfigMode::Auto
                            } else {
                                ConfigMode::Manual
                            };
                            this.config.update(cx, |c, _| {
                                c.mode = this.mode.clone();
                                let _ = save(c);
                            });
                            cx.notify();
                        })),
                ),
            )
            .child(match self.mode {
                ConfigMode::Auto => self.auto.clone().into_any_element(),
                ConfigMode::Manual => self.manual.clone().into_any_element(),
            })
    }
}
