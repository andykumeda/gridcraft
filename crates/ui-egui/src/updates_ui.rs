//! One background check at startup (and on Help ▸ Check for Updates…) and the modal prompt that
//! offers the new release: Update and Restart (direct self-update), Download or Later. The flow
//! itself is `craft_update::updater::Updater`.

use craft_update::updater::{Action, Status, Updater};

use crate::SheetApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) struct Runtime {
    updater: Updater,
    disabled: bool,
    /// The check at start was asked for; `tick` starts it with the context.
    startup: bool,
    /// Help ▸ Check for Updates… was chosen; the next `tick` starts it.
    manual: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&crate::UPDATE_APP, VERSION), disabled: false, startup: false, manual: false }
    }
}

impl SheetApp {
    /// Uses only explicitly configured assistant-enabled releases; never falls back upstream.
    /// Call when attaching the independently installed assistant bridge. An explicit update
    /// opt-out remains in force.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn configure_assistant_updates(&mut self, bridge_protocol: u32) {
        let root = craft_ai_native::storage::SharedStore::default_root().map_err(|error| error.to_string());
        self.updates_rt.updater = craft_update::custom::updater(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION"), root, bridge_protocol);
    }

    /// Separately packaged previews must not install standard releases over themselves.
    pub fn disable_updates(&mut self) {
        self.updates_rt.disabled = true;
        self.updates_rt.startup = false;
        self.updates_rt.manual = false;
    }
}

/// Ask for a check; the next frame's `tick` starts it. A manual one reports every outcome.
pub(crate) fn start(app: &mut SheetApp, manual: bool) {
    if app.updates_rt.disabled {
        if manual {
            app.message = Some((
                "GridCraft".into(),
                "Software updates are disabled for this preview. Install a new Assistant Preview package to update it.".into(),
            ));
        }
        return;
    }
    if manual {
        app.updates_rt.manual = true;
    } else {
        app.updates_rt.startup = true;
    }
}

pub(crate) fn tick(app: &mut SheetApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if rt.disabled {
        return;
    }
    if std::mem::take(&mut rt.manual) {
        rt.updater.check_now(ctx);
    } else if rt.startup {
        rt.updater.check_at_start(ctx);
    }
    rt.updater.tick(ctx);
    let lang = app.ui.language;
    let outcome = match rt.updater.status() {
        Status::UpToDate => Some(lang.tr("GridCraft {version} is up to date.").replace("{version}", VERSION)),
        Status::Failed(e) => Some(lang.tr("Could not check for updates: {error}").replace("{error}", e)),
        Status::Idle | Status::Checking => None,
    };
    if let Some(message) = outcome {
        let message = match rt.updater.repository() {
            Some(repo) => format!("{message}\n{}", lang.tr("Update source: {repo}").replace("{repo}", repo)),
            None => message,
        };
        rt.updater.dismiss_status();
        app.message = Some(("GridCraft".into(), message));
    }
}

fn can_restart(app: &SheetApp) -> bool {
    !app.session.documents().iter().any(|d| d.is_dirty()) && crate::assistant::restart_safe(app)
}

pub(crate) fn show(app: &mut SheetApp, ctx: &egui::Context) {
    if app.updates_rt.disabled {
        return;
    }
    // The helper is scheduled before close: workbook edits and in-flight assistant input/turns must both be safe.
    let clean = can_restart(app);
    match app.updates_rt.updater.show_if(ctx, clean) {
        Action::OpenUrl(url) => {
            if let Some(open) = &app.services.open_url {
                open(&url);
            }
        }
        // The guarded close path asks about unsaved work; the restart helper relaunches once the
        // process has exited.
        Action::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        Action::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_blocks_startup_and_manual_updates() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        assert!(!app.updates_rt.disabled);
        start(&mut app, false);
        start(&mut app, true);
        assert!(app.updates_rt.startup && app.updates_rt.manual);
        app.disable_updates();
        app.configure_assistant_updates(1);
        assert!(!app.updates_rt.startup && !app.updates_rt.manual);
        start(&mut app, false);
        start(&mut app, true);
        let ctx = egui::Context::default();
        tick(&mut app, &ctx);
        show(&mut app, &ctx);
        assert!(!app.updates_rt.startup && !app.updates_rt.manual);
        assert!(matches!(app.updates_rt.updater.status(), Status::Idle));
        assert!(app.message.as_ref().is_some_and(|(_, text)| text.contains("disabled")));
    }
    #[test]
    fn app_restart_requires_idle_assistant_and_clean_workbooks() {
        let mut app = SheetApp::new(gridcraft_engine::Session::new(), Default::default());
        assert!(can_restart(&app));
        app.assistant.widget.busy = true;
        assert!(!can_restart(&app), "a clean workbook does not permit interrupting a chat");
        app.assistant.widget.busy = false;
        app.assistant.widget.settings.busy = true;
        assert!(!can_restart(&app), "assistant credential/settings work must finish");
        app.assistant.widget.settings.busy = false;
        assert!(can_restart(&app));
        app.session.new_workbook();
        app.run("cell.set", serde_json::json!({"cell":"A1","input":"synthetic unsaved edit"})).unwrap();
        assert!(!can_restart(&app), "the existing workbook safeguard remains in force");
    }
}
