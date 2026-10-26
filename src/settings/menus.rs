use crate::actions::{Action, CellType, key_label};
use crate::config::{AppConfig, Item, PieMenu, edit};
use crate::{assets, key};
use gpui_kit::component::accordion::{Accordion, AccordionItem};
use gpui_kit::component::button::ButtonVariants;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Disableable, Icon, IconName, scroll::ScrollableElement, switch::Switch};
use gpui_kit::*;
use rdev::Key;
use zopra::prelude::*;
use zopra::ui::{Tab, TabBar};

/// node kinds a menu can hold, in tab order
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Command,
    App,
    Macro,
}

impl Kind {
    fn placeholder(self) -> &'static str {
        match self {
            Self::Command => "e.g. echo \"hello world\"",
            Self::App if cfg!(windows) => "e.g. C:\\Program Files\\App\\app.exe",
            Self::App if cfg!(target_os = "macos") => "e.g. /Applications/App.app",
            Self::App => "e.g. /usr/bin/app",
            Self::Macro => "",
        }
    }
}

/// inline create / edit form bound to a single menu
#[derive(Clone, PartialEq)]
struct Form {
    menu: usize,
    edit: Option<usize>,
    kind: Kind,
    show_terminal: bool,
    recording: bool,
    keys: Vec<Key>,
    label: Entity<InputState>,
    detail: Entity<InputState>,
    delay: Entity<InputState>,
}

impl Form {
    fn new(config: &Entity<AppConfig>, menu: usize, edit: Option<usize>, window: &mut Window, cx: &mut App) -> Self {
        let item = edit.map(|ix| config.read(cx).menus[menu].items[ix].clone());
        let (kind, show_terminal, detail, keys, delay) = match item.as_ref().and_then(|item| item.action.clone()) {
            Some(Action::Command { cmd, show_terminal }) => (Kind::Command, show_terminal, cmd.to_string(), vec![], 50),
            Some(Action::App { path }) => (Kind::App, false, path.to_string_lossy().into(), vec![], 50),
            Some(Action::Macro { keys, delay }) => (Kind::Macro, false, String::new(), keys, delay),
            None => (Kind::Command, false, String::new(), vec![], 50),
        };
        let (label, delay) = item.map_or_else(Default::default, |item| (item.label.to_string(), delay.to_string()));
        let mut input = |placeholder: &'static str, value: String| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).default_value(value))
        };
        Self {
            menu,
            edit,
            kind,
            show_terminal,
            recording: false,
            keys,
            label: input("Name", label),
            detail: input(kind.placeholder(), detail),
            delay: input("delay ms", delay),
        }
    }

    fn action(&self, cx: &App) -> Action {
        let detail = self.detail.read(cx).value();
        match self.kind {
            Kind::Command => Action::Command { cmd: detail, show_terminal: self.show_terminal },
            Kind::App => Action::App { path: detail.as_ref().into() },
            Kind::Macro => Action::Macro {
                keys: self.keys.clone(),
                delay: self.delay.read(cx).value().trim().parse().unwrap_or(50),
            },
        }
    }

    fn invalid(&self, cx: &App) -> bool {
        match self.kind {
            Kind::Macro => self.recording || self.keys.is_empty(),
            _ => self.detail.read(cx).value().is_empty(),
        }
    }
}

/// changes the open form, if any
fn with_form(form: Signal<Option<Form>>, cx: &mut App, change: impl FnOnce(&mut Form, &mut App)) {
    if let Some(mut state) = form.get(cx) {
        change(&mut state, cx);
        form.set(Some(state), cx);
    }
}

fn set_kind(form: Signal<Option<Form>>, kind: Kind, window: &mut Window, cx: &mut App) {
    with_form(form, cx, |form, cx| {
        if form.recording {
            form.keys = key::stop_recording();
            form.recording = false;
        }
        // the detail keeps its text but takes the new kind's placeholder
        let value = form.detail.read(cx).value();
        form.detail = cx.new(|cx| InputState::new(window, cx).placeholder(kind.placeholder()).default_value(value));
        form.kind = kind;
    });
}

fn toggle_recording(form: Signal<Option<Form>>, cx: &mut App) {
    with_form(form, cx, |form, _| {
        form.recording = !form.recording;
        form.keys = if form.recording {
            key::start_recording();
            Vec::new()
        } else {
            key::stop_recording()
        };
    });
}

/// fills the app path from a file picker
fn browse(detail: &Entity<InputState>, window: &mut Window, cx: &mut App) {
    let paths = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: None });
    let detail = detail.clone();
    window
        .spawn(cx, async move |cx| {
            if let Ok(Ok(Some(mut paths))) = paths.await
                && let Some(path) = paths.pop()
            {
                _ = detail.update_in(cx, |state, window, cx| state.set_value(path.to_string_lossy().to_string(), window, cx));
            }
        })
        .detach();
}

/// stores the form's node in its menu and closes the form
fn save(form: Signal<Option<Form>>, config: &Entity<AppConfig>, cx: &mut App) {
    let Some(state) = form.get(cx) else { return };
    form.set(None, cx);
    key::stop_recording();
    let action = state.action(cx);
    if let Action::App { path } = &action {
        // pull the exe icon in the background
        let path = path.clone();
        cx.background_spawn(async move { assets::extract_icon(&path) }).detach();
    }
    let label = Some(state.label.read(cx).value()).filter(|label| !label.is_empty());
    let item = Item {
        label: label.unwrap_or_else(|| state.detail.read(cx).value()),
        action: Some(action),
        celltype: CellType::Normal,
    };
    edit(config, cx, |config| match state.edit {
        Some(ix) => config.menus[state.menu].items[ix] = item,
        None => config.menus[state.menu].items.push(item),
    });
}

#[component]
fn node_form(form: Signal<Option<Form>>, state: Form, config: Entity<AppConfig>) {
    let invalid = state.invalid(cx);
    let detail: Entity<InputState> = state.detail.clone();
    let recording = state.recording;
    let (border, fg) = (cx.theme().border, cx.theme().foreground);

    view! {
        <div class="flex flex-col gap-2 py-1">
            <TabBar segmented class="self-start" selected_index={state.kind as usize}
                on_click={|index: &usize, window: &mut Window, cx: &mut App| {
                    set_kind(form, [Kind::Command, Kind::App, Kind::Macro][*index], window, cx)
                }}
            >
                <Tab label="Command" />
                <Tab label="App" />
                <Tab label="Macro" />
            </TabBar>
            <input state={&state.label} />
            {...(state.kind == Kind::Macro).then(|| view! {
                <div class="flex flex-col gap-2">
                    <div class="flex flex-row items-center gap-2">
                        {if recording {
                            view! { <button danger label="Stop recording" on_click={|| toggle_recording(form, cx)} /> }
                        } else {
                            view! { <button outline label="Record keys" on_click={|| toggle_recording(form, cx)} /> }
                        }}
                        <input state={&state.delay} w={px(96.0)} />
                    </div>
                    <div class="flex flex-row flex-wrap items-center gap-1">
                        {...key::recorded().iter().map(|key| view! {
                            <div class="px-2 py-0.5 rounded-sm border" border_color={border} text_color={fg}>{key_label(key)}</div>
                        })}
                    </div>
                </div>
            })}
            {...(state.kind != Kind::Macro).then(|| view! { <input state={&detail} /> })}
            {...(state.kind == Kind::Command).then(|| {
                Switch::new("show-terminal")
                    .checked(state.show_terminal)
                    .label("Show terminal")
                    .on_click(move |checked: &bool, _: &mut Window, cx: &mut App| with_form(form, cx, |form, _| form.show_terminal = *checked))
            })}
            <div class="flex flex-row items-center justify-end gap-2">
                {...(state.kind == Kind::App).then(|| view! {
                    <button outline label="Browse..." on_click={|_, window, cx| browse(&detail, window, cx)} />
                })}
                <button outline label="Test" disabled={invalid} on_click={|| if let Some(state) = form() { state.action(cx).run() }} />
                <button outline label="Cancel" on_click={|| { key::stop_recording(); form(None) }} />
                <button primary label={if state.edit.is_some() { "Save" } else { "Create" }} disabled={invalid}
                    on_click={|| save(form, &config, cx)}
                />
            </div>
        </div>
    }
}

#[component]
pub fn menus_editor(config: Entity<AppConfig>) {
    let (open, set_open) = use_state(Some(0usize));
    let (form, set_form) = use_state(None::<Form>);
    let menus = config.read(cx).menus.clone();
    let (fg, muted, hover) = (cx.theme().foreground, cx.theme().muted_foreground, cx.theme().secondary);

    // one name input per menu, rebuilt whenever a menu is added or removed
    let names = window.use_keyed_state(("menu-names", menus.len()), cx, |window, cx| {
        let inputs = menus.iter().enumerate().map(|(ix, menu)| {
            let input = cx.new(|cx| InputState::new(window, cx).default_value(menu.name.clone()));
            let config = config.clone();
            let subscription = cx.subscribe(&input, move |_, input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    let name = input.read(cx).value();
                    edit(&config, cx, |config| {
                        if let Some(menu) = config.menus.get_mut(ix) {
                            menu.name = name;
                        }
                    });
                }
            });
            (input, subscription)
        });
        inputs.collect::<Vec<_>>()
    });

    let mut accordion = Accordion::new("menus")
        .multiple(false)
        .h_auto()
        .on_toggle_click(move |open: &[usize], _: &mut Window, cx: &mut App| set_open(open.first().copied()));
    for (mi, menu) in menus.iter().enumerate() {
        let name = names.read(cx)[mi].0.clone();
        let mut item = AccordionItem::new().open(open() == Some(mi)).icon(Icon::new(IconName::ChartPie)).title(view! {
            <div class="flex flex-row items-center gap-2">
                <input state={&name} w={px(140.0)} />
                <div class="text-sm" text_color={muted}>{format!("{} nodes", menu.items.len())}</div>
            </div>
        });
        for (ni, node) in menu.items.iter().enumerate() {
            item = item.child(view! {
                <div id={("node-row", ni)} class="flex flex-row items-center justify-between px-2 py-1 rounded-sm" hover={|style| style.bg(hover)}>
                    <div class="flex flex-row items-center gap-2">
                        {assets::node_icon(node.action.as_ref(), fg)}
                        {node.label.clone()}
                    </div>
                    <div class="flex flex-row items-center gap-2">
                        <button key={ni} outline compact icon={IconName::Settings}
                            on_click={|_, window, cx| set_form(Some(Form::new(&config, mi, Some(ni), window, cx)))}
                        />
                        <button key={ni} danger compact icon={IconName::Delete}
                            on_click={|| edit(&config, cx, |config| _ = config.menus[mi].items.remove(ni))}
                        />
                    </div>
                </div>
            });
        }
        let editing = form().filter(|state| state.menu == mi);
        let item = item.children(editing.map(|state| view! { <NodeForm form={form} state={state} config={config.clone()} /> })).child(view! {
            <div class="flex flex-row items-center justify-between">
                <button key={mi} outline label="Add node" icon={IconName::Plus}
                    on_click={|_, window, cx| set_form(Some(Form::new(&config, mi, None, window, cx)))}
                />
                <button key={mi} danger ghost compact icon={IconName::Delete}
                    on_click={|| {
                        edit(&config, cx, |config| _ = config.menus.remove(mi));
                        set_open(None);
                        set_form(None);
                    }}
                />
            </div>
        });
        accordion = accordion.item(|_| item);
    }

    view! {
        <div id="menus-editor" class="flex flex-col size-full p-4 gap-3" overflow_y_scrollbar>
            {accordion}
            <button outline label="New menu" icon={IconName::Plus}
                on_click={|| {
                    edit(&config, cx, |config| {
                        let name = format!("MENU {}", config.menus.len() + 1).into();
                        config.menus.push(PieMenu { name, items: Vec::new() });
                    });
                    set_open(Some(config.read(cx).menus.len() - 1));
                    set_form(None);
                }}
            />
        </div>
    }
}
