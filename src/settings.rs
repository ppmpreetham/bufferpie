mod menus;

use crate::config::{AppConfig, Colors, ConfigMode, PieMenu, edit};
use gpui_kit::component::input::{Editor, EditorState, InputEvent};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::{ActiveTheme, Colorize, Icon, Root};
use gpui_kit::*;
use menus::MenusEditor;
use zopra::prelude::*;
use zopra::ui::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem, Tab, TabBar};

/// the open settings window, if any
fn window(cx: &App) -> Option<WindowHandle<Root>> {
    cx.windows().into_iter().find_map(|window| window.downcast::<Root>())
}

/// repaints the settings window, e.g. while recording keys
pub fn refresh(cx: &mut App) {
    if let Some(window) = window(cx) {
        _ = window.update(cx, |_, window, _| window.refresh());
    }
}

/// opens the settings window, or focuses the one already open
pub fn open(config: Entity<AppConfig>, cx: &mut App) {
    if let Some(window) = window(cx) {
        _ = window.update(cx, |_, window, _| window.activate_window());
        return;
    }
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::centered(size(px(720.0), px(480.0)), cx)),
        titlebar: Some(TitlebarOptions { title: Some("Settings".into()), ..Default::default() }),
        kind: WindowKind::Normal,
        is_resizable: true,
        ..Default::default()
    };
    _ = gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| SettingsWindow(config)));
}

struct SettingsWindow(Entity<AppConfig>);

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        view! { <SettingsView config={self.0.clone()} /> }
    }
}

#[component]
fn settings_view(config: Entity<AppConfig>) {
    let (appearance, set_appearance) = use_state(true);
    let fg = cx.theme().foreground;
    let icon = |path: &'static str| Icon::empty().path(path).text_color(fg);

    view! {
        <div class="size-full flex flex-row bg-theme-background text-theme-foreground">
            <Sidebar class="w-[180px]">
                <SidebarGroup label="">
                    <SidebarMenu>
                        <SidebarMenuItem label="Appearance" icon={icon("logos/appearence.svg")} active={appearance()} on_click={|| set_appearance(true)} />
                        <SidebarMenuItem label="Config" icon={icon("logos/config.svg")} active={!appearance()} on_click={|| set_appearance(false)} />
                    </SidebarMenu>
                </SidebarGroup>
            </Sidebar>
            <div class="flex-1 h-full overflow-hidden">
                {if appearance() {
                    view! { <AppearanceTab config={config} /> }.into_any_element()
                } else {
                    view! { <ConfigTab config={config} /> }.into_any_element()
                }}
            </div>
        </div>
    }
}

/// the themeable colors, by label
type Slot = fn(&mut Colors) -> &mut u32;

const SLOTS: [(&str, Slot); 4] = [
    ("Arc", |colors| &mut colors.arc),
    ("Surface", |colors| &mut colors.surface),
    ("Surface Hover", |colors| &mut colors.surface_hover),
    ("Text", |colors| &mut colors.text),
];

#[component]
fn appearance_tab(config: Entity<AppConfig>) {
    view! {
        <div id="appearance-tab" class="size-full overflow-y-scroll flex flex-col gap-3 p-4">
            {...SLOTS.map(|(label, slot)| view! { <ColorRow label={label} slot={slot} config={config.clone()} /> })}
        </div>
    }
}

#[component]
fn color_row(label: &'static str, slot: Slot, config: Entity<AppConfig>) {
    let picker: Entity<ColorPickerState> = window.use_state(cx, |window, cx| {
        let color = *slot(&mut config.read(cx).colors.clone());
        ColorPickerState::new(window, cx).default_value(rgb(color))
    });
    use_event(&picker, |event: &ColorPickerEvent| {
        if let ColorPickerEvent::Change(Some(color)) = event {
            let value = u32::from_str_radix(color.to_hex().trim_start_matches('#'), 16).unwrap_or(0);
            edit(&config, cx, |config| *slot(&mut config.colors) = value);
        }
    });

    view! {
        <div class="flex flex-row justify-between items-center gap-4">
            <div>{label}</div>
            {ColorPicker::new(&picker)}
        </div>
    }
}

#[component]
fn config_tab(config: Entity<AppConfig>) {
    let auto = config.read(cx).mode == ConfigMode::Auto;

    view! {
        <div class="flex flex-col gap-2 size-full">
            <TabBar segmented class="self-start m-2" selected_index={usize::from(!auto)}
                on_click={|index: &usize, _: &mut Window, cx: &mut App| {
                    let mode = if *index == 0 { ConfigMode::Auto } else { ConfigMode::Manual };
                    edit(&config, cx, |config| config.mode = mode);
                }}
            >
                <Tab label="Auto" />
                <Tab label="Manual" />
            </TabBar>
            <div class="flex-1 w-full overflow-hidden">
                {if auto {
                    view! { <MenusEditor config={config} /> }.into_any_element()
                } else {
                    view! { <ManualEditor config={config} /> }.into_any_element()
                }}
            </div>
        </div>
    }
}

/// the menus as raw json, applied whenever they parse
#[component]
fn manual_editor(config: Entity<AppConfig>) {
    let editor: Entity<EditorState> =
        window.use_state(cx, |window, cx| EditorState::new(window, cx).language("json").line_number(true));
    let (error, set_error) = use_state(None::<SharedString>);
    let json = serde_json::to_string_pretty(&config.read(cx).menus).unwrap_or_default();

    // menus edited elsewhere (the auto tab) are pushed into the editor
    use_effect!(|| {
        if editor.read(cx).value() != json {
            editor.update(cx, |editor, cx| editor.set_value(json, _window, cx));
        }
    }, [json.clone()]);
    use_event(&editor, |event: &InputEvent| {
        if matches!(event, InputEvent::Change) {
            match serde_json::from_str::<Vec<PieMenu>>(&editor.read(cx).value()) {
                Ok(menus) => {
                    set_error(None);
                    edit(&config, cx, |config| config.menus = menus);
                }
                Err(err) => set_error(Some(err.to_string().into())),
            }
        }
    });

    view! {
        <div class="flex flex-col gap-2 p-4 size-full">
            {Editor::new(&editor).h_full()}
            {...error().map(|err| view! { <div text_color={rgb(0xf38ba8)}>{err}</div> })}
        </div>
    }
}
