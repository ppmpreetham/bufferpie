use gpui::*;
use gpui_component::{WindowExt, button::Button, v_flex};

use crate::ui::pie_menu::Item;

pub fn open_macro_creator(
    window: &mut Window,
    cx: &mut App,
    _on_create: impl Fn(Item, &mut Window, &mut App) + 'static + Clone,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("New Macro")
            .child(v_flex().gap_3().child("Macro recording is coming soon."))
            .footer(
                Button::new("close")
                    .outline()
                    .label("Close")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
    });
    // TODO: keystroke recording from src/key.rs,
}
