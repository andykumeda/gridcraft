//! GridCraft's host adapter for the reusable Craft assistant.

use craft_ui::assistant::{Action, State};
use serde_json::{Value, json};

use crate::SheetApp;

#[cfg(not(target_arch = "wasm32"))]
mod host;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub struct Assistant {
    pub open: bool,
    pub widget: State,
    pub enabled: bool,
    pub(crate) ctx: Option<egui::Context>,
    pub(crate) effective: String,
    #[cfg(not(target_arch = "wasm32"))]
    native: native::Native,
}

impl Default for Assistant {
    fn default() -> Self {
        Self {
            open: false,
            widget: State::default(),
            enabled: cfg!(not(target_arch = "wasm32")),
            ctx: None,
            effective: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            native: native::Native::default(),
        }
    }
}

impl Drop for Assistant {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.native.cancel();
    }
}

pub fn commands() -> Vec<Value> {
    vec![
        json!({"id":"ui.assistant","label":"Assistant","location":"View › Assistant","params":"value?: bool","enabled":true}),
        json!({"id":"ui.assistant.send","label":"Send assistant prompt","location":"Agents","params":"text: string","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.newChat","label":"New assistant chat","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.stop","label":"Stop assistant","location":"Agents","params":"{}","enabled":true}),
        json!({"id":"ui.assistant.status","label":"Assistant status","location":"Agents","params":"{}","enabled":true}),
    ]
}

pub(crate) fn command(app: &mut SheetApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if !id.starts_with("ui.assistant") {
        return None;
    }
    let result = match id {
        "ui.assistant" => {
            app.assistant.open = params.get("value").and_then(Value::as_bool).unwrap_or(!app.assistant.open);
            ensure_loaded(app);
            Ok(json!({"open":app.assistant.open}))
        }
        "ui.assistant.send" => {
            let Some(text) = params.get("text").and_then(Value::as_str).filter(|s| !s.trim().is_empty()) else {
                return Some(Err("give a nonempty text prompt".into()));
            };
            if params.get("files").is_some() {
                return Some(Err("Assistant file access is unavailable in this preview".into()));
            }
            app.assistant.open = true;
            action(app, Action::Send(text.into())).map(|()| json!({"started":true}))
        }
        "ui.assistant.newChat" => action(app, Action::NewChat).map(|()| json!({"started":true})),
        "ui.assistant.stop" => action(app, Action::Stop).map(|()| json!({"stopped":true})),
        "ui.assistant.status" => Ok(
            json!({"open":app.assistant.open,"busy":app.assistant.widget.busy,"available":app.assistant.enabled,"settingsBusy":app.assistant.widget.settings.busy,"chatId":app.assistant.widget.active_chat,"model":app.assistant.widget.chat_route,"chatModel":app.assistant.widget.chat_route,"controlModel":app.assistant.widget.chat_route,"status":app.assistant.widget.status,"error":app.assistant.widget.error}),
        ),
        _ => Err("unknown assistant command".into()),
    };
    Some(result)
}

pub fn pane(app: &mut SheetApp, ui: &mut egui::Ui) {
    app.assistant.ctx = Some(ui.ctx().clone());
    ensure_loaded(app);
    ui.small("Uses the active workbook. Assistant file access is unavailable.");
    if app.assistant.widget.settings_open {
        ui.small(&app.assistant.effective);
    }
    if !app.assistant.widget.busy {
        app.assistant.widget.target_label = app.session.active().map(|d| d.display_title()).unwrap_or_else(|| "No workbook".into());
    }
    let id = egui::Id::unique(("assistant", app.session.active_window()));
    let language = app.ui.language;
    let actions = craft_ui::assistant::show(ui, id, &mut app.assistant.widget, |text| language.tr(text).into());
    apply_actions(app, actions);
}

pub fn settings(app: &mut SheetApp, ui: &mut egui::Ui) {
    app.assistant.ctx = Some(ui.ctx().clone());
    ensure_loaded(app);
    if !app.assistant.enabled {
        ui.label("The assistant is available in the desktop app.");
        return;
    }
    let id = egui::Id::unique(("assistant_settings", app.session.active_window()));
    let language = app.ui.language;
    ui.small(&app.assistant.effective);
    let actions = craft_ui::assistant::settings(ui, id, &mut app.assistant.widget.settings, |text| language.tr(text).into());
    apply_actions(app, actions);
}

fn apply_actions(app: &mut SheetApp, actions: Vec<Action>) {
    for requested in actions {
        if let Err(error) = action(app, requested) {
            app.assistant.widget.error = Some(error.clone());
            app.assistant.widget.settings.error = Some(error);
        }
    }
}

fn ensure_loaded(app: &mut SheetApp) {
    #[cfg(not(target_arch = "wasm32"))]
    native::ensure_loaded(app);
    #[cfg(target_arch = "wasm32")]
    let _ = app;
}

fn action(app: &mut SheetApp, action: Action) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::action(app, action)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, action);
        Err("The assistant is available in the desktop app.".into())
    }
}

pub(crate) fn poll(app: &mut SheetApp, ctx: &egui::Context) {
    app.assistant.ctx = Some(ctx.clone());
    #[cfg(not(target_arch = "wasm32"))]
    native::poll(app, ctx);
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
