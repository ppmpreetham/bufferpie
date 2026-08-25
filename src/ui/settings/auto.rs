use super::creators::{open_app_creator, open_command_creator, open_macro_creator};
use super::save;
use crate::ui::config::AppConfig;
use crate::ui::pie_menu::{Item, PieMenu};
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};

pub struct AutoEditor {
    pub config: Entity<AppConfig>,
}

impl AutoEditor {
    pub fn new(config: Entity<AppConfig>) -> Self {
        Self { config }
    }

    fn add_item(&mut self, item: Item, cx: &mut Context<Self>) {
        self.config.update(cx, |c, cx| {
            if c.menus.is_empty() {
                c.menus.push(PieMenu {
                    name: "MENU".into(),
                    items: vec![],
                });
            }
            c.menus[0].items.push(item);
            let _ = save(c);
            cx.notify();
        });
    }

    fn delete_item(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.config.update(cx, |c, cx| {
            if let Some(menu) = c.menus.first_mut()
                && ix < menu.items.len()
            {
                menu.items.remove(ix);
            }
            let _ = save(c);
            cx.notify();
        });
    }
}

impl Render for AutoEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let items: Vec<Item> = self
            .config
            .read(cx)
            .menus
            .iter()
            .flat_map(|m| m.items.clone())
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .children(items.iter().enumerate().map(|(ix, item)| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0x202020))
                    .child(item.label.clone())
                    .child(
                        Button::new(("delete-item", ix))
                            .danger()
                            .compact()
                            .label("Delete")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.delete_item(ix, cx);
                            })),
                    )
            }))
            .child({
                let this = this.clone();
                Button::new("create-component")
                    .primary()
                    .label("Create")
                    .dropdown_menu(move |menu, _, _| {
                        let this_cmd = this.clone();
                        let this_app = this.clone();
                        menu.item(
                            PopupMenuItem::new("Command").on_click(move |_, window, cx| {
                                let this_cmd = this_cmd.clone();
                                open_command_creator(window, cx, move |item, _, cx| {
                                    this_cmd.update(cx, |this, cx| this.add_item(item, cx));
                                });
                            }),
                        )
                        .item(PopupMenuItem::new("App").on_click(move |_, window, cx| {
                            let this_app = this_app.clone();
                            open_app_creator(window, cx, move |item, _, cx| {
                                this_app.update(cx, |this, cx| this.add_item(item, cx));
                            });
                        }))
                        .item(PopupMenuItem::new("Macro").on_click(move |_, window, cx| {
                            open_macro_creator(window, cx, |_, _, _| {});
                        }))
                    })
            })
    }
}
