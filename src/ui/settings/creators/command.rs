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
        cmd::run_command,
        types::{Action, CellType},
    },
    ui::pie_menu::Item,
};

pub fn open_command_creator(
    window: &mut Window,
    cx: &mut App,
    on_create: impl Fn(Item, &mut Window, &mut App) + 'static + Clone,
) {
    let label_input = cx.new(|cx| InputState::new(window, cx).placeholder("Label"));
    let command_input = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. code ."));

    window.open_dialog(cx, move |dialog, _, _| {
        let label_input = label_input.clone();
        let command_input = command_input.clone();
        let test_input = command_input.clone();
        let on_create = on_create.clone();

        dialog
            .title("New Command")
            .child(
                v_flex()
                    .gap_3()
                    .child(Input::new(&label_input))
                    .child(Input::new(&command_input)),
            )
            .footer(
                h_flex().gap_2().children(vec![
                    Button::new("test")
                        .outline()
                        .label("Test")
                        .on_click(move |_, _, cx| {
                            run_command(&test_input.read(cx).value());
                        })
                        .into_any_element(),
                    Button::new("cancel")
                        .outline()
                        .label("Cancel")
                        .on_click(|_, window, cx| window.close_dialog(cx))
                        .into_any_element(),
                    Button::new("create")
                        .primary()
                        .label("Create")
                        .on_click(move |_, window, cx| {
                            let label: SharedString =
                                label_input.read(cx).value().to_string().into();
                            let command: SharedString =
                                command_input.read(cx).value().to_string().into();
                            on_create(
                                Item {
                                    label,
                                    action: Some(Action::Command(command)),
                                    celltype: CellType::Normal,
                                },
                                window,
                                cx,
                            );
                            window.close_dialog(cx);
                        })
                        .into_any_element(),
                ]),
            )
    });
}
