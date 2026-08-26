use super::appearance::AppearanceTab;
use super::config_tab::ConfigTab;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
};

#[derive(Clone, Copy, PartialEq)]
enum SidebarTab {
    Appearance,
    Config,
}

pub struct SettingsView {
    tab: SidebarTab,
    appearance_tab: Entity<AppearanceTab>,
    config_tab: Entity<ConfigTab>,
}

impl SettingsView {
    pub fn new(appearance_tab: Entity<AppearanceTab>, config_tab: Entity<ConfigTab>) -> Self {
        Self {
            tab: SidebarTab::Appearance,
            appearance_tab,
            config_tab,
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = self.tab;
        let dialog_layer = gpui_component::Root::render_dialog_layer(window, cx);

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                Sidebar::new("settings-sidebar").w(px(180.0)).child(
                    SidebarGroup::new("").child(
                        SidebarMenu::new()
                            .child(
                                SidebarMenuItem::new("Appearance")
                                    .icon(
                                        Icon::empty()
                                            .path("logos/appearence.svg")
                                            .text_color(cx.theme().foreground),
                                    )
                                    .active(tab == SidebarTab::Appearance)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.tab = SidebarTab::Appearance;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                SidebarMenuItem::new("Config")
                                    .icon(
                                        Icon::empty()
                                            .path("logos/config.svg")
                                            .text_color(cx.theme().foreground),
                                    )
                                    .active(tab == SidebarTab::Config)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.tab = SidebarTab::Config;
                                        cx.notify();
                                    })),
                            ),
                    ),
                ),
            )
            .child(div().flex_1().h_full().child(match self.tab {
                SidebarTab::Appearance => self.appearance_tab.clone().into_any_element(),
                SidebarTab::Config => self.config_tab.clone().into_any_element(),
            }))
            .children(dialog_layer)
    }
}
