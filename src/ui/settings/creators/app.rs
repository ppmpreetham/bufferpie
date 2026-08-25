use std::{path::PathBuf, rc::Rc};

use gpui::*;
use gpui_component::{
    WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

use crate::{
    actions::{
        app_open::open_app,
        types::{Action, CellType},
    },
    ui::pie_menu::Item,
};

pub struct AppCreator {
    label_input: Entity<InputState>,
    path_input: Entity<InputState>,
    on_create: Rc<dyn Fn(Item, &mut Window, &mut App)>,
}

impl AppCreator {
    fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path_input = self.path_input.clone();
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
                let _ = path_input.update_in(cx, |state, window, cx| {
                    state.set_value(path.to_string_lossy().to_string(), window, cx);
                });
            }
        })
        .detach();
    }

    fn test(&mut self, cx: &mut Context<Self>) {
        open_app(&self.path_input.read(cx).value());
    }

    fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let label = self.label_input.read(cx).value().to_string().into();
        let path = PathBuf::from(self.path_input.read(cx).value().to_string());
        (self.on_create)(
            Item {
                label,
                action: Some(Action::App { path }),
                celltype: CellType::Normal,
            },
            window,
            cx,
        );

        window.close_dialog(cx);
    }
}

impl Render for AppCreator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Input::new(&self.label_input))
            .child(Input::new(&self.path_input))
            .child(
                Button::new("browse")
                    .outline()
                    .label("Browse...")
                    .on_click(cx.listener(|this, _, window, cx| this.browse(window, cx))),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("test")
                            .outline()
                            .label("Test")
                            .on_click(cx.listener(|this, _, _, cx| this.test(cx))),
                    )
                    .child(
                        Button::new("create")
                            .primary()
                            .label("Create")
                            .on_click(cx.listener(|this, _, window, cx| this.create(window, cx))),
                    ),
            )
    }
}

pub fn open_app_creator(
    window: &mut Window,
    cx: &mut App,
    on_create: impl Fn(Item, &mut Window, &mut App) + 'static,
) {
    let label_input = cx.new(|cx| InputState::new(window, cx).placeholder("Label"));
    let path_input = cx.new(|cx| InputState::new(window, cx).placeholder("No app selected"));
    let on_create = std::rc::Rc::new(on_create);

    let creator = cx.new(|_| AppCreator {
        label_input,
        path_input,
        on_create,
    });

    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title("New App").child(creator.clone())
    });
}
