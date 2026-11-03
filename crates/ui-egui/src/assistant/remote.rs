//! Assistant tool calls use the same MCP server and live control dispatcher as external agents.
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::Duration;

use craft_ai_core::AiError;
use craft_ai_core::assistant::{RuntimeConfig, Target};
use craft_assistant::controller::{Controller, Request};
use craft_assistant::host_proxy::{Event, Proxy};
use craft_assistant::protocol::StartTurn;
use craft_ui::assistant::Action;
use serde_json::json;

use super::host::{self, Guard, McpHost, Pending};
use crate::SheetApp;

#[derive(Default)]
pub(super) struct Remote {
    pub enabled: bool,
    controller: Option<Controller>,
    turn: Option<Turn>,
    pub config: Option<RuntimeConfig>,
    dispatching: bool,
    stop_requested: bool,
    label: String,
    settings: Option<bool>,
    shown: bool,
    keyboard_visible: bool,
    language: String,
    generation: u64,
}
struct Turn {
    guard: Guard,
    proxy: Proxy<Pending>,
    target: Target,
    prompt: String,
    files: Vec<String>,
    started: bool,
    saw_busy: bool,
    catalog: Option<StartTurn>,
}

impl Remote {
    pub(super) fn cancel(&self) {
        if let Some(turn) = &self.turn {
            turn.proxy.cancel();
        }
    }
    pub(super) fn owns_keyboard(&self) -> bool {
        self.keyboard_visible && self.controller.as_ref().is_some_and(Controller::owns_keyboard)
    }
}

pub(super) fn ensure_loaded(app: &mut SheetApp) {
    if app.assistant.remote.controller.is_none() {
        app.assistant.remote.controller = Some(Controller::new("gridcraft", BTreeMap::new()));
        app.assistant.enabled = true;
        app.assistant.widget.available = true;
    }
}

pub(super) fn show(app: &mut SheetApp, ui: &mut egui::Ui, settings: bool) {
    ensure_loaded(app);
    ui.small(app.ui.language.tr("Can open, save, and control GridCraft workbooks and windows."));
    app.assistant.remote.shown = true;
    let label = app.session.active().map(|doc| doc.display_title()).unwrap_or_else(|| app.ui.language.tr("No workbook").into());
    let language = app.ui.language.code();
    if let Some(controller) = &mut app.assistant.remote.controller {
        if app.assistant.remote.generation != controller.generation() {
            app.assistant.remote.generation = controller.generation();
            app.assistant.remote.label.clear();
            app.assistant.remote.settings = None;
            app.assistant.remote.language.clear();
        }
        if app.assistant.remote.language != language && controller.ready() && controller.language_changed().is_ok() {
            app.assistant.remote.language = language.to_owned();
        }
        if app.assistant.remote.label != label && controller.command("context", json!(label)).is_ok() {
            app.assistant.remote.label = label;
        }
        if app.assistant.remote.settings != Some(settings) && controller.command("settings", json!(settings)).is_ok() {
            app.assistant.remote.settings = Some(settings);
        }
        if let Some(error) = &app.assistant.widget.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        controller.show(ui);
    }
}

pub(super) fn action(app: &mut SheetApp, action: Action) -> Result<(), String> {
    ensure_loaded(app);
    match action {
        Action::Stop => stop(app),
        Action::Send(prompt) => {
            let result = begin(app, prompt.clone(), app.assistant.widget.authorized_files.clone());
            if result.is_err() && app.assistant.widget.draft.is_empty() {
                app.assistant.widget.draft = prompt;
            }
            result
        }
        Action::ChooseFiles => choose_files(app),
        action => {
            let controller = app.assistant.remote.controller.as_mut().ok_or("The shared assistant is unavailable")?;
            match action {
                Action::NewChat => controller.command("new_chat", json!({})),
                Action::OpenChat(id) => controller.command("open_chat", json!(id)),
                Action::DeleteChat(id) => controller.command("delete_chat", json!(id)),
                Action::AddFile(path) => controller.command("add_file", json!(path)),
                Action::ReloadSettings => controller.command("reload_settings", json!({})),
                _ => Err("Use the shared assistant settings to change assistant configuration.".into()),
            }
        }
    }
}

fn begin(app: &mut SheetApp, prompt: String, files: Vec<String>) -> Result<(), String> {
    if app.assistant.remote.turn.is_some() || app.assistant.remote.dispatching {
        return Err("Finish or stop the current assistant turn first.".into());
    }
    if let Some(controller) = &mut app.assistant.remote.controller {
        controller.dismiss_error();
    }
    if prompt.trim().is_empty() || files.len() > 32 {
        return Err("Give a nonempty prompt and at most 32 exact file paths.".into());
    }
    let wake = app.assistant.ctx.clone().ok_or("The assistant needs an active app window")?;
    let guard = Guard::new(app, prompt.clone(), files.clone());
    let target = guard.target();
    let proxy = Proxy::spawn(target.clone(), wake.clone(), |sender, cancel, wake, config| {
        McpHost::new(sender, cancel, wake, Duration::from_secs(config.request_timeout_secs))
    })?;
    app.assistant.widget.busy = true;
    app.assistant.widget.error = None;
    app.assistant.remote.turn = Some(Turn { guard, proxy, target, prompt, files, started: false, saw_busy: false, catalog: None });
    if let Some(controller) = &mut app.assistant.remote.controller {
        controller.ensure_started(&wake);
    }
    Ok(())
}

fn stop(app: &mut SheetApp) -> Result<(), String> {
    if app.assistant.remote.dispatching {
        app.assistant.remote.stop_requested = true;
    }
    app.assistant.remote.cancel();
    app.assistant.remote.turn = None;
    app.assistant.widget.busy = false;
    if let Some(controller) = &mut app.assistant.remote.controller
        && controller.ready()
    {
        return controller.command("stop", json!({}));
    }
    Ok(())
}

pub(super) fn poll(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(controller) = &mut app.assistant.remote.controller else { return };
    if app.assistant.open {
        controller.ensure_started(ctx);
    }
    if !app.assistant.open && !app.assistant.remote.shown {
        controller.hidden();
    }
    app.assistant.remote.keyboard_visible = app.assistant.remote.shown;
    app.assistant.remote.shown = false;
    controller.poll(ctx);
    let busy = controller.status.busy;
    let requests = controller.requests();
    for request in requests {
        match request {
            Request::BeginTurn { prompt, files } => {
                if let Err(error) = begin(app, prompt, files) {
                    app.assistant.widget.error = Some(error.clone());
                    if let Some(controller) = &mut app.assistant.remote.controller {
                        let _ = controller.command("reject_turn", json!(error));
                    }
                }
            }
            Request::Stop => {
                if let Err(error) = stop(app) {
                    app.assistant.widget.error = Some(error);
                }
            }
            Request::ChooseFiles => {
                if let Err(error) = choose_files(app) {
                    app.assistant.widget.error = Some(error);
                }
            }
            Request::Translate { keys } => {
                let values = keys
                    .into_iter()
                    .map(|key| {
                        let translated = app.ui.language.tr(&key).to_owned();
                        (key, translated)
                    })
                    .collect();
                if let Some(controller) = &mut app.assistant.remote.controller {
                    let _ = controller.translations(values);
                }
            }
            Request::Tool { id, request } => {
                let result = match &mut app.assistant.remote.turn {
                    Some(turn) if turn.started => turn.proxy.submit(id.clone(), request),
                    _ => Err(AiError::invalid("No active app-authorized assistant turn; the action was not executed.")),
                };
                if let Err(error) = result
                    && let Some(controller) = &mut app.assistant.remote.controller
                {
                    let _ = controller.tool_result(id, Err(error));
                }
            }
        }
    }
    if let Some(mut turn) = app.assistant.remote.turn.take() {
        app.assistant.remote.dispatching = true;
        while let Ok(pending) = turn.proxy.pending.try_recv() {
            host::dispatch(app, ctx, &mut turn.guard, pending);
            if app.assistant.remote.stop_requested {
                turn.proxy.cancel();
                break;
            }
        }
        app.assistant.remote.dispatching = false;
        let stopped = std::mem::take(&mut app.assistant.remote.stop_requested);
        let mut failed = None;
        loop {
            match turn.proxy.events.try_recv() {
                Ok(Event::Catalog { config, tools, initial_read }) => {
                    turn.proxy.set_limits(&config);
                    app.assistant.remote.config = Some(config);
                    turn.catalog =
                        Some(StartTurn { prompt: turn.prompt.clone(), target: turn.target.clone(), tools, files: turn.files.clone(), initial_read });
                }
                Ok(Event::Result { id, result }) => {
                    if let Some(controller) = &mut app.assistant.remote.controller
                        && let Err(error) = controller.tool_result(id, result)
                    {
                        failed = Some(error);
                        break;
                    }
                }
                Ok(Event::Failed(error)) => {
                    failed = Some(error);
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    failed = Some("The app tool worker stopped; completed edits will not be replayed.".into());
                    break;
                }
            }
        }
        if let Some(controller) = &mut app.assistant.remote.controller
            && !turn.started
            && controller.ready()
            && !controller.status.settings_busy
            && !controller.status.model.is_empty()
            && let Some(request) = turn.catalog.take()
        {
            match controller.start_turn(request) {
                Ok(()) => turn.started = true,
                Err(error) => failed = Some(error),
            }
        }
        turn.saw_busy |= busy;
        let controller_error = app
            .assistant
            .remote
            .controller
            .as_ref()
            .and_then(|controller| {
                controller.error().or({
                    if controller.status.model.is_empty() && !controller.status.settings_busy { controller.status.error.as_deref() } else { None }
                })
            })
            .map(str::to_owned);
        if stopped {
            turn.proxy.cancel();
        } else if let Some(error) = failed.or(controller_error) {
            turn.proxy.cancel();
            app.assistant.widget.error = Some(error);
            if let Some(controller) = &mut app.assistant.remote.controller {
                let _ = controller.command("stop", json!({}));
            }
        } else if !turn.started || !turn.saw_busy || busy {
            app.assistant.remote.turn = Some(turn);
        }
    }
    if let Some(controller) = &app.assistant.remote.controller {
        let status = &controller.status;
        app.assistant.widget.busy = status.busy || app.assistant.remote.turn.is_some();
        app.assistant.widget.settings.busy = status.settings_busy;
        app.assistant.widget.active_chat = status.chat_id.clone();
        app.assistant.widget.authorized_files = status.authorized_files.clone();
        app.assistant.widget.chat_route = status.model.clone();
        app.assistant.widget.control_route = status.model.clone();
        app.assistant.widget.status = status.status.clone();
        if let Some(error) = controller.error().or(status.error.as_deref()) {
            app.assistant.widget.error = Some(error.into());
        }
    }
    if app.assistant.remote.turn.is_some() || app.assistant.remote.dispatching {
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

fn choose_files(app: &mut SheetApp) -> Result<(), String> {
    let pick = app.services.pick_open.as_ref().ok_or("The file chooser is unavailable; enter an exact file path in the assistant.")?;
    if let Some(path) = pick() {
        app.assistant.remote.controller.as_mut().ok_or("The shared assistant is not ready")?.command("add_file", json!(path))?;
    }
    Ok(())
}

pub(super) fn update_status(app: &SheetApp) -> serde_json::Value {
    app.assistant.remote.controller.as_ref().map(Controller::update_status).unwrap_or_else(|| json!({"ready":false,"version":null}))
}

pub(super) fn service_command(app: &mut SheetApp, name: &str) -> Result<serde_json::Value, String> {
    if !app.assistant.remote.enabled {
        return Err("This app has not enabled the independently installed assistant.".into());
    }
    ensure_loaded(app);
    if name == "ui.assistant.updateStatus" {
        return Ok(update_status(app));
    }
    let ctx = app.assistant.ctx.clone().ok_or("The assistant needs an active app window")?;
    if name == "ui.assistant.reloadComponent" && (app.assistant.remote.turn.is_some() || app.assistant.remote.dispatching) {
        return Err("Finish or stop the current assistant turn before reloading its component.".into());
    }
    let controller = app.assistant.remote.controller.as_mut().ok_or("The shared assistant is unavailable")?;
    match name {
        "ui.assistant.updateCheck" => controller.check_for_updates(&ctx)?,
        "ui.assistant.updateInstall" => controller.install_available_update(&ctx)?,
        "ui.assistant.reloadComponent" => {
            controller.reload_component(&ctx)?;
            app.assistant.widget.error = None;
        }
        _ => return Err("Unknown shared assistant service command".into()),
    }
    Ok(json!({"accepted":true}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helper_grant_removal_replaces_old_automation_grants_without_starting_a_process() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        app.assistant.remote.enabled = true;
        ensure_loaded(&mut app);
        app.assistant.widget.authorized_files = vec!["/synthetic/previous-grant".into()];
        let ctx = egui::Context::default();
        poll(&mut app, &ctx);
        assert!(app.assistant.widget.authorized_files.is_empty());
        assert!(!app.assistant.remote.controller.as_ref().unwrap().ready());
        assert!(!app.assistant.remote.owns_keyboard());
    }
    #[test]
    fn nested_begin_and_reload_are_rejected_while_turn_is_taken_for_gui_dispatch() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        app.assistant.remote.enabled = true;
        app.assistant.remote.dispatching = true;
        let error = begin(&mut app, "Nested request".into(), vec![]).unwrap_err();
        assert!(error.contains("Finish or stop"), "{error}");
        assert!(app.assistant.remote.turn.is_none());
        assert!(app.assistant.remote.controller.is_none(), "rejected nested requests must not start a helper");
        app.assistant.ctx = Some(egui::Context::default());
        let error = service_command(&mut app, "ui.assistant.reloadComponent").unwrap_err();
        assert!(error.contains("Finish or stop"), "{error}");
        assert!(!app.assistant.remote.controller.as_ref().unwrap().ready());
    }

    #[test]
    fn self_stop_during_gui_dispatch_sets_deferred_cancellation_without_starting_a_turn() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        app.assistant.remote.dispatching = true;
        app.assistant.widget.busy = true;
        assert!(!app.assistant.remote.stop_requested);
        stop(&mut app).unwrap();
        // The taken turn is owned by poll; it consumes this flag to cancel instead of reinserting it.
        assert!(app.assistant.remote.stop_requested);
        assert!(app.assistant.remote.turn.is_none());
        assert!(!app.assistant.widget.busy);
        assert!(app.assistant.remote.controller.is_none());
    }
}
