//! GridCraft's host adapter for the reusable Craft assistant.

use craft_ui::assistant::{Action, State};
use serde_json::{Value, json};

use crate::SheetApp;

#[cfg(not(target_arch = "wasm32"))]
mod host;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod remote;

pub struct Assistant {
    pub open: bool,
    pub widget: State,
    pub enabled: bool,
    pub(crate) ctx: Option<egui::Context>,
    pub(crate) effective: String,
    #[cfg(not(target_arch = "wasm32"))]
    native: native::Native,
    #[cfg(not(target_arch = "wasm32"))]
    remote: remote::Remote,
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
            #[cfg(not(target_arch = "wasm32"))]
            remote: remote::Remote::default(),
        }
    }
}

impl Drop for Assistant {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.native.cancel();
        #[cfg(not(target_arch = "wasm32"))]
        self.remote.cancel();
    }
}

impl SheetApp {
    /// Attaches the independently installed assistant and its explicit app-update source.
    /// Desktop entry points call this once before startup update checks.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn enable_shared_assistant(&mut self) {
        self.assistant.native.cancel();
        self.assistant.remote.enabled = true;
        self.assistant.enabled = true;
        self.assistant.widget.available = true;
        self.configure_assistant_updates(craft_assistant::config::compatibility().protocol);
    }
}

pub(crate) fn owns_keyboard(app: &SheetApp) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.assistant.remote.enabled && app.assistant.remote.owns_keyboard()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        false
    }
}

pub fn commands() -> Vec<Value> {
    vec![
        json!({"id":"ui.assistant","label":"Assistant","location":"View › Assistant","params":"value?: bool","enabled":true}),
        json!({"id":"ui.assistant.send","label":"Send assistant prompt","location":"Agents","params":"text: string; files?: array of exact paths","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.newChat","label":"New assistant chat","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.stop","label":"Stop assistant","location":"Agents","params":"{}","enabled":true}),
        json!({"id":"ui.assistant.status","label":"Assistant status","location":"Agents","params":"{}","enabled":true}),
        json!({"id":"ui.assistant.updateCheck","label":"Check shared assistant updates","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.updateInstall","label":"Install offered shared assistant update","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.updateStatus","label":"Shared assistant update status","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
        json!({"id":"ui.assistant.reloadComponent","label":"Reload shared assistant component","location":"Agents","params":"{}","enabled":cfg!(not(target_arch="wasm32"))}),
    ]
}

pub(crate) fn command(app: &mut SheetApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if !id.starts_with("ui.assistant") {
        return None;
    }
    if matches!(id, "ui.assistant.updateCheck" | "ui.assistant.updateInstall" | "ui.assistant.updateStatus" | "ui.assistant.reloadComponent") {
        if !params.as_object().is_some_and(serde_json::Map::is_empty) {
            return Some(Err("This assistant service command takes an empty parameter object.".into()));
        }
        #[cfg(not(target_arch = "wasm32"))]
        return Some(remote::service_command(app, id));
        #[cfg(target_arch = "wasm32")]
        return Some(Err("The shared assistant is available in the desktop app.".into()));
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
            if app.assistant.widget.busy {
                return Some(Err("finish or stop the current assistant turn first".into()));
            }
            if let Some(files) = params.get("files") {
                let Some(files) = files.as_array().filter(|files| files.len() <= 32) else {
                    return Some(Err("files must contain at most 32 exact file paths".into()));
                };
                let Some(files) = files.iter().map(|file| file.as_str().map(str::to_owned)).collect::<Option<Vec<_>>>() else {
                    return Some(Err("file paths must be strings".into()));
                };
                if let Err(error) = set_files(app, files) {
                    return Some(Err(error));
                }
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
    #[cfg(not(target_arch = "wasm32"))]
    if app.assistant.remote.enabled {
        app.assistant.ctx = Some(ui.ctx().clone());
        remote::show(app, ui, false);
        return;
    }
    app.assistant.ctx = Some(ui.ctx().clone());
    ensure_loaded(app);
    ui.small("Can open, save, and control GridCraft workbooks and windows.");
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
    #[cfg(not(target_arch = "wasm32"))]
    if app.assistant.remote.enabled {
        app.assistant.ctx = Some(ui.ctx().clone());
        remote::show(app, ui, true);
        return;
    }
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

fn set_files(app: &mut SheetApp, files: Vec<String>) -> Result<(), String> {
    if files.len() > 32 {
        return Err("give at most 32 exact file paths".into());
    }
    if files.iter().any(|file| !std::path::Path::new(file).is_absolute()) {
        return Err("use absolute file paths".into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    if files.iter().any(|file| std::path::Path::new(file).is_dir()) {
        return Err("choose individual files, not folders".into());
    }
    app.assistant.widget.authorized_files = files;
    Ok(())
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
    if app.assistant.remote.enabled {
        remote::ensure_loaded(app);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    native::ensure_loaded(app);
    #[cfg(target_arch = "wasm32")]
    let _ = app;
}

fn action(app: &mut SheetApp, action: Action) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    if app.assistant.remote.enabled {
        return remote::action(app, action);
    }
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
    #[cfg(not(target_arch = "wasm32"))]
    if app.assistant.remote.enabled {
        remote::poll(app, ctx);
        return;
    }
    app.assistant.ctx = Some(ctx.clone());
    #[cfg(not(target_arch = "wasm32"))]
    native::poll(app, ctx);
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod shared_component_commands {
    use super::*;
    #[test]
    fn update_controls_require_activation_and_report_status_without_starting_a_helper() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        for id in ["ui.assistant.updateCheck", "ui.assistant.updateInstall", "ui.assistant.reloadComponent"] {
            assert!(command(&mut app, id, &json!({})).unwrap().is_err());
        }
        app.enable_shared_assistant();
        assert!(app.assistant.remote.enabled);
        assert!(command(&mut app, "ui.assistant.updateStatus", &json!({})).unwrap().is_ok());
        assert!(command(&mut app, "ui.assistant.updateStatus", &json!({"source":"untrusted"})).unwrap().is_err());
        assert!(command(&mut app, "ui.assistant.updateCheck", &json!({})).unwrap().unwrap_err().contains("active app window"));
    }
}
