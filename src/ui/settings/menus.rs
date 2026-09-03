use std::path::PathBuf;

use super::save;
use crate::actions::{
    app_open::open_app,
    cmd::run_command,
    keystrokes::{key_label, run_keystrokes},
    types::{Action, CellType},
};
use crate::key;
use crate::ui::assets::{NodeIcon, node_icon};
use crate::ui::config::AppConfig;
use crate::ui::pie_menu::{Item, PieMenu};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    accordion::{Accordion, AccordionItem},
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    switch::Switch,
    tab::{Tab, TabBar},
    v_flex,
};

/// node kinds a menu can hold
#[derive(Clone, Copy, PartialEq)]
enum NodeKind {
    Command,
    App,
    Macro,
}

impl NodeKind {
    const ALL: [Self; 3] = [Self::Command, Self::App, Self::Macro];

    fn label(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::App => "App",
            Self::Macro => "Macro",
        }
    }
}

/// inline create form bound to a single menu
struct NodeForm {
    menu_ix: usize,
    kind: NodeKind,
    show_terminal: bool,
    recording: bool,
    keys: Vec<rdev::Key>,
    label: Entity<InputState>,
    detail: Entity<InputState>,
    delay: Entity<InputState>,
}

pub struct MenusEditor {
    config: Entity<AppConfig>,
    open: Option<usize>,
    form: Option<NodeForm>,
}

impl MenusEditor {
    pub fn new(config: Entity<AppConfig>) -> Self {
        Self {
            config,
            open: Some(0),
            form: None,
        }
    }

    fn persist(&mut self, edit: impl FnOnce(&mut AppConfig), cx: &mut Context<Self>) {
        self.config.update(cx, |c, cx| {
            edit(c);
            _ = save(c);
            cx.notify();
        });
    }

    fn add_menu(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.persist(
            |c| {
                c.menus.push(PieMenu {
                    name: format!("MENU {}", c.menus.len() + 1).into(),
                    items: Vec::new(),
                })
            },
            cx,
        );
        self.open = Some(self.config.read(cx).menus.len() - 1);
        self.form = None;
        cx.notify();
    }

    fn remove_menu(&mut self, menu_ix: usize, cx: &mut Context<Self>) {
        self.persist(
            |c| {
                c.menus.remove(menu_ix);
            },
            cx,
        );
        self.open = None;
        self.form = None;
    }

    fn remove_node(&mut self, menu_ix: usize, node_ix: usize, cx: &mut Context<Self>) {
        self.persist(
            |c| {
                c.menus[menu_ix].items.remove(node_ix);
            },
            cx,
        );
    }

    fn open_form(&mut self, menu_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let mut input = |placeholder: &'static str| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        self.form = Some(NodeForm {
            menu_ix,
            kind: NodeKind::Command,
            show_terminal: false,
            recording: false,
            keys: Vec::new(),
            label: input("Label"),
            detail: input("e.g. code ."),
            delay: input("delay ms"),
        });
        cx.notify();
    }

    fn set_kind(&mut self, ix: &usize, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(form) = self.form.as_mut() {
            if form.recording {
                form.keys = key::stop_recording();
                form.recording = false;
            }
            form.kind = NodeKind::ALL[*ix];
        }
        cx.notify();
    }

    fn close_form(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        key::stop_recording();
        self.form = None;
        cx.notify();
    }

    fn toggle_recording(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.form.as_mut() else {
            return;
        };
        form.recording = !form.recording;
        form.keys = if form.recording {
            key::start_recording();
            Vec::new()
        } else {
            key::stop_recording()
        };
        cx.notify();
    }

    fn test_node(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.form.as_ref() else {
            return;
        };
        let detail = form.detail.read(cx).value().to_string();
        match form.kind {
            NodeKind::Command => run_command(&detail, form.show_terminal),
            NodeKind::App => open_app(&detail),
            NodeKind::Macro => {
                // replay the captured keys so they can be checked before saving
                let delay = form.delay.read(cx).value().trim().parse().unwrap_or(50);
                run_keystrokes(&form.keys, delay);
            }
        }
    }

    fn browse(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(detail) = self.form.as_ref().map(|f| f.detail.clone()) else {
            return;
        };
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(mut paths))) = rx.await
                && let Some(path) = paths.pop()
            {
                _ = detail.update_in(cx, |state, window, cx| {
                    state.set_value(path.to_string_lossy().to_string(), window, cx);
                });
            }
        })
        .detach();
    }

    fn create_node(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.form.take() else { return };
        key::stop_recording();
        let detail = form.detail.read(cx).value().to_string();
        let label = {
            let label = form.label.read(cx).value().to_string();
            if label.is_empty() {
                detail.clone()
            } else {
                label
            }
        };
        let (action, exe) = match form.kind {
            NodeKind::Command => (
                Action::Command {
                    cmd: detail.into(),
                    show_terminal: form.show_terminal,
                },
                None,
            ),
            NodeKind::App => (
                Action::App {
                    path: PathBuf::from(&detail),
                },
                Some(PathBuf::from(&detail)),
            ),
            // TODO: keystroke recording polish
            NodeKind::Macro if form.recording || form.keys.is_empty() => return,
            NodeKind::Macro => (
                Action::Macro {
                    keys: form.keys,
                    delay: form.delay.read(cx).value().trim().parse().unwrap_or(50),
                },
                None,
            ),
        };

        let menu_ix = form.menu_ix;
        self.persist(
            |c| {
                c.menus[menu_ix].items.push(Item {
                    label: label.into(),
                    action: Some(action),
                    celltype: CellType::Normal,
                })
            },
            cx,
        );

        // pull the exe icon in the background, repaint once it lands
        if let Some(exe) = exe {
            let extract = cx.background_spawn(async move { crate::ui::assets::extract_icon(&exe) });
            cx.spawn_in(window, async move |this, cx| {
                extract.await;
                _ = this.update_in(cx, |_, _, _| {});
            })
            .detach();
        }
    }

    fn render_form(&mut self, menu_ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(form) = self.form.as_ref() else {
            return div().into_any_element();
        };
        let (kind, label, detail, show_terminal, recording, captured) = (
            form.kind,
            &form.label,
            &form.detail,
            form.show_terminal,
            form.recording,
            &form.keys,
        );
        let has_detail = !detail.read(cx).value().is_empty();
        let invalid = match kind {
            NodeKind::Macro => recording || captured.is_empty(),
            _ => !has_detail,
        };

        let form_ui = v_flex()
            .gap_2()
            .py_1()
            .child(
                TabBar::new(("node-kind", menu_ix))
                    .segmented()
                    .self_start()
                    .selected_index(kind as usize)
                    .children(NodeKind::ALL.map(|k| Tab::new().label(k.label())))
                    .on_click(cx.listener(Self::set_kind)),
            )
            .child(Input::new(label))
            .when(kind == NodeKind::Macro, |d| {
                d.child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(if recording {
                                    Button::new(("stop-recording", menu_ix))
                                        .danger()
                                        .label("Stop recording")
                                        .on_click(cx.listener(Self::toggle_recording))
                                } else {
                                    Button::new(("start-recording", menu_ix))
                                        .outline()
                                        .label("Record keys")
                                        .on_click(cx.listener(Self::toggle_recording))
                                })
                                .child(Input::new(&form.delay).w(px(96.0))),
                        )
                        .child(h_flex().flex_wrap().gap_1().children(
                            key::recorded().into_iter().map(|k| {
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .text_color(cx.theme().foreground)
                                    .child(key_label(k))
                            }),
                        )),
                )
            })
            .when(kind != NodeKind::Macro, |d| d.child(Input::new(detail)))
            .when(kind == NodeKind::Command, |d| {
                let this = cx.entity();
                d.child(
                    Switch::new(("show-terminal", menu_ix))
                        .checked(show_terminal)
                        .label("Show terminal")
                        .on_click(move |checked, _, cx| {
                            this.update(cx, |this, cx| {
                                if let Some(form) = this.form.as_mut() {
                                    form.show_terminal = *checked;
                                }
                                cx.notify();
                            });
                        }),
                )
            })
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .when(kind == NodeKind::App, |d| {
                        d.child(
                            Button::new(("browse", menu_ix))
                                .outline()
                                .label("Browse...")
                                .on_click(cx.listener(Self::browse)),
                        )
                    })
                    .child(
                        Button::new(("test", menu_ix))
                            .outline()
                            .label("Test")
                            .disabled(invalid)
                            .on_click(cx.listener(Self::test_node)),
                    )
                    .child(
                        Button::new(("cancel", menu_ix))
                            .outline()
                            .label("Cancel")
                            .on_click(cx.listener(Self::close_form)),
                    )
                    .child(
                        Button::new(("create", menu_ix))
                            .primary()
                            .label("Create")
                            .disabled(invalid)
                            .on_click(cx.listener(Self::create_node)),
                    ),
            );
        form_ui.into_any_element()
    }
}

impl Render for MenusEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let menus = self.config.read(cx).menus.clone();

        let mut accordion = Accordion::new("menus").multiple(false);
        for (mi, menu) in menus.iter().enumerate() {
            let expanded = self.open == Some(mi);
            let mut item = AccordionItem::new()
                .icon(Icon::new(IconName::ChartPie))
                .title(
                    h_flex().gap_2().child(menu.name.clone()).child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} nodes", menu.items.len())),
                    ),
                );

            for (ni, node) in menu.items.iter().enumerate() {
                item = item.child(
                    h_flex()
                        .id(("node-row", ni))
                        .justify_between()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .hover(|s| s.bg(cx.theme().secondary))
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(match node_icon(node.action.as_ref()) {
                                    NodeIcon::Svg(path) => svg()
                                        .path(path)
                                        .size(px(16.0))
                                        .text_color(cx.theme().foreground)
                                        .into_any_element(),
                                    NodeIcon::File(path) => {
                                        img(path).size(px(16.0)).into_any_element()
                                    }
                                })
                                .child(node.label.clone()),
                        )
                        .child(
                            Button::new(("del-node", ni))
                                .danger()
                                .compact()
                                .icon(IconName::Delete)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.remove_node(mi, ni, cx)),
                                ),
                        ),
                );
            }

            item = item
                .children(match self.form.as_ref().map(|f| f.menu_ix) {
                    Some(ix) if ix == mi => Some(self.render_form(mi, cx)),
                    _ => None,
                })
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            Button::new(("add-node", mi))
                                .outline()
                                .label("Add node")
                                .icon(IconName::Plus)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_form(mi, window, cx)
                                })),
                        )
                        .child(
                            Button::new(("del-menu", mi))
                                .danger()
                                .ghost()
                                .compact()
                                .icon(IconName::Delete)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.remove_menu(mi, cx)),
                                ),
                        ),
                );
            accordion = accordion.item(|_| item.open(expanded));
        }

        v_flex()
            .id("menus-editor")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(accordion.flex_1())
            .child(
                Button::new("add-menu")
                    .outline()
                    .label("New menu")
                    .icon(IconName::Plus)
                    .on_click(cx.listener(Self::add_menu)),
            )
    }
}
