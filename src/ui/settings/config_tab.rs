use super::{ConfigMode, manual::ManualEditor, menus::MenusEditor, save};
use crate::ui::config::AppConfig;
use gpui::*;
use gpui_component::{
    tab::{Tab, TabBar},
    v_flex,
};

pub struct ConfigTab {
    pub mode: ConfigMode,
    pub auto: Entity<MenusEditor>,
    pub manual: Entity<ManualEditor>,
    pub config: Entity<AppConfig>,
}

impl Render for ConfigTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_auto = matches!(self.mode, ConfigMode::Auto);

        v_flex()
            .gap_2()
            .size_full()
            .child(
                v_flex().items_start().p_2().child(
                    TabBar::new("config-mode")
                        .segmented()
                        .self_start()
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
                                _ = save(c);
                            });
                            cx.notify();
                        })),
                ),
            )
            .child(div().flex_1().w_full().overflow_hidden().child(match self.mode {
                ConfigMode::Auto => self.auto.clone().into_any_element(),
                ConfigMode::Manual => self.manual.clone().into_any_element(),
            }))
    }
}
