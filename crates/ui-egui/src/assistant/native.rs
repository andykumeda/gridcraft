// Adapted from WordCraft; provider, credential and persistence services remain shared.
macro_rules! tl {
    ($text:expr) => {
        $text
    };
}

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use craft_ai_core::assistant::{AssistantRole, Conversation, Message, MessageRole, RuntimeConfig, Target, tool_model};
use craft_ai_core::{
    AiError, AppId, Auth, Binding, Connection, ConnectionDraft, ConnectionId, KeyChange, ModelId, ModelInfo, Prepared, SecretRef, SecretString,
    StoreFile,
};
use craft_ai_native::codex::{CodexOptions, CodexProvider};
use craft_ai_native::secrets::{KeyringStore, SecretStore};
use craft_ai_native::storage::{HistoryStore, SharedStore};
use craft_ai_native::{Cancellation, ChatProvider, HttpProvider, ModelRequest, TurnEvent, TurnRequest, resolve_choice, run_turn};
use craft_ui::assistant::{Action, Activity, ChatSummary};
use serde_json::Value;

use super::host::{self, Guard, McpHost, Pending};
use crate::SheetApp;

#[derive(Default)]
pub(super) struct Native {
    initialized: bool,
    root: Option<PathBuf>,
    codex_template: Option<(PathBuf, Arc<CodexProvider>)>,
    conversation: Option<Conversation>,
    job: Option<mpsc::Receiver<Result<Update, String>>>,
    job_cancel: Cancellation,
    turn: Option<Turn>,
}

struct Turn {
    events: mpsc::Receiver<WorkerEvent>,
    calls: mpsc::Receiver<Pending>,
    cancel: Cancellation,
    guard: Guard,
    fresh_text: bool,
    pending_activity: Vec<(String, usize)>,
}

impl Native {
    fn codex_for_turn(&mut self, root: &std::path::Path) -> CodexProvider {
        if let Some((cached_root, template)) = &self.codex_template
            && cached_root == root
        {
            return template.fork_turn();
        }
        let template = Arc::new(CodexProvider::new(CodexOptions::in_shared_root(root)));
        let turn = template.fork_turn();
        self.codex_template = Some((root.to_owned(), template));
        turn
    }

    fn invalidate_codex_on(&mut self, action: &Action) {
        if matches!(action, Action::ReloadSettings | Action::BeginCodexSignIn) {
            self.codex_template = None;
        }
    }

    pub(super) fn cancel(&self) {
        self.job_cancel.cancel();
        if let Some(turn) = &self.turn {
            turn.cancel.cancel();
        }
    }
}

/// Screenshot fixtures simulate native availability without opening the user's settings or chats.
#[cfg(test)]
pub(super) fn enable_without_profile(app: &mut SheetApp) {
    app.assistant.native.initialized = true;
    app.assistant.enabled = true;
}

enum WorkerEvent {
    Runtime(TurnEvent),
    Saved(Conversation),
}

enum Update {
    Loaded { root: PathBuf, store: StoreFile, history: Vec<ChatSummary>, config: RuntimeConfig },
    Store(StoreFile),
    ConnectionSaved { change: ConnectionChange, id: ConnectionId },
    ConnectionDeleted { change: ConnectionChange, id: ConnectionId },
    Models { draft: ConnectionDraft, models: Vec<ModelInfo> },
    Chat(Conversation),
    Deleted(String),
    Login { url: String, code: String },
    Notice(String),
}

fn app_id() -> Result<AppId, String> {
    AppId::new("gridcraft").map_err(|e| e.to_string())
}
fn fail(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn history(store: &SharedStore, app: &AppId) -> Result<HistoryStore, String> {
    HistoryStore::new(store.root(), app).map_err(fail)
}
fn summaries(store: &HistoryStore) -> Result<Vec<ChatSummary>, String> {
    Ok(store.list().map_err(fail)?.into_iter().map(|c| ChatSummary { id: c.id, title: c.title, interrupted: c.interrupted }).collect())
}

fn job(app: &mut SheetApp, work: impl FnOnce(mpsc::Sender<Result<Update, String>>, Cancellation) + Send + 'static) -> Result<(), String> {
    if app.assistant.native.job.is_some() {
        return Err("wait for the current assistant settings operation".into());
    }
    let (sender, receiver) = mpsc::channel();
    let cancel = Cancellation::new();
    let worker_cancel = cancel.clone();
    let wake = app.assistant.ctx.clone();
    std::thread::Builder::new()
        .name("assistant-settings".into())
        .spawn(move || {
            work(sender, worker_cancel);
            if let Some(wake) = wake {
                wake.request_repaint();
            }
        })
        .map_err(fail)?;
    app.assistant.native.job = Some(receiver);
    app.assistant.native.job_cancel = cancel;
    app.assistant.widget.settings.busy = true;
    app.assistant.widget.settings.error = None;
    app.assistant.widget.error = None;
    Ok(())
}

pub(super) fn ensure_loaded(app: &mut SheetApp) {
    app.assistant.widget.available = app.assistant.enabled;
    if !app.assistant.enabled || app.assistant.native.initialized {
        return;
    }
    app.assistant.native.initialized = true;
    if let Err(error) = job(app, |sender, _| {
        let result = (|| {
            let root = SharedStore::default_root().map_err(fail)?;
            let store = SharedStore::new(root.clone());
            let history = summaries(&history(&store, &app_id()?)?)?;
            Ok(Update::Loaded { root, store: store.load().map_err(fail)?, history, config: store.config().map_err(fail)? })
        })();
        let _ = sender.send(result);
    }) {
        app.assistant.widget.error = Some(error);
    }
}

struct Providers {
    http: HttpProvider,
    codex: CodexProvider,
}
impl Providers {
    fn new(root: &std::path::Path, secrets: Arc<dyn SecretStore>) -> Self {
        Self { http: HttpProvider::new(secrets), codex: CodexProvider::new(CodexOptions::in_shared_root(root)) }
    }
}
impl ChatProvider for Providers {
    fn complete(&self, request: &ModelRequest<'_>, cancel: &Cancellation, emit: &mut dyn FnMut(&str)) -> Result<Message, AiError> {
        if request.connection.provider == "codex" { self.codex.complete(request, cancel, emit) } else { self.http.complete(request, cancel, emit) }
    }
    fn models(&self, connection: &Connection, config: &RuntimeConfig, cancel: &Cancellation) -> Result<Vec<ModelInfo>, AiError> {
        if connection.provider == "codex" { self.codex.models(connection, config, cancel) } else { self.http.models(connection, config, cancel) }
    }
}

pub(super) fn action(app: &mut SheetApp, requested: Action) -> Result<(), String> {
    ensure_loaded(app);
    if matches!(requested, Action::Stop) {
        app.assistant.native.cancel();
        app.assistant.widget.status = Some(tl!("Stopping; completed edits remain undoable.").into());
        return Ok(());
    }
    if !app.assistant.enabled {
        return Err("The assistant is available in the desktop app.".into());
    }
    if app.assistant.widget.busy {
        return Err("stop or finish the current assistant turn first".into());
    }
    app.assistant.native.invalidate_codex_on(&requested);
    if matches!(requested, Action::ReloadSettings) {
        if app.assistant.native.job.is_some() {
            return Err("wait for the current assistant settings operation".into());
        }
        app.assistant.native.initialized = false;
        ensure_loaded(app);
        return Ok(());
    }
    if matches!(requested, Action::AddFile(_) | Action::RemoveFile(_) | Action::ChooseFiles) {
        return Err("This preview works with the active workbook only; assistant file access is unavailable.".into());
    }
    if let Action::Send(prompt) = requested {
        let result = app
            .assistant
            .native
            .root
            .clone()
            .ok_or_else(|| "assistant settings are loading; try again when ready".to_string())
            .and_then(|root| send(app, root, prompt.clone()));
        if result.is_err() && app.assistant.widget.draft.is_empty() {
            app.assistant.widget.draft = prompt;
        }
        return result;
    }
    let root = app.assistant.native.root.clone().ok_or("assistant settings are loading; try again when ready")?;
    let app_id = app_id()?;
    job(app, move |sender, cancel| {
        let store = SharedStore::new(root);
        if matches!(requested, Action::BeginCodexSignIn) {
            let result = login(&store, &sender, &cancel);
            let _ = sender.send(result);
        } else {
            let _ = sender.send(settings_action(&store, &app_id, requested, &cancel));
        }
    })
}

fn turn_request(shared: &SharedStore, app: AppId, messages: Vec<Message>, target: Target) -> Result<TurnRequest, String> {
    let config = shared.config().map_err(fail)?;
    let choice = resolve_choice(&shared.load().map_err(fail)?, &app).map_err(fail)?;
    Ok(TurnRequest { app, messages, target, chat: choice.clone(), control: choice, config })
}

fn send(app: &mut SheetApp, root: PathBuf, prompt: String) -> Result<(), String> {
    if app.assistant.native.job.is_some() {
        return Err("wait for assistant settings to finish loading".into());
    }
    app.settle_drafts()?;
    let app_id = app_id()?;
    let wake = app.assistant.ctx.clone().ok_or("the assistant needs an active app window")?;
    let target =
        Target { id: app.session.active().map(|d| d.uid).unwrap_or(0).to_string(), revision: app.session.active().map(|d| d.revision).unwrap_or(0) };
    let mut conversation = app.assistant.native.conversation.clone();
    let mut messages = conversation.as_ref().map(|c| c.messages.clone()).unwrap_or_default();
    messages.push(Message::text(MessageRole::User, prompt.clone()));
    let (events_tx, events) = mpsc::channel();
    let (call_tx, calls) = mpsc::channel();
    let cancel = Cancellation::new();
    let worker_cancel = cancel.clone();
    let request = turn_request(&SharedStore::new(root.clone()), app_id.clone(), messages.clone(), target)?;
    let title: String = prompt.chars().take(80).collect();
    let mut guard = Guard::new(app, prompt, Vec::new());
    guard.set_limits(&request.config);
    let codex = app.assistant.native.codex_for_turn(&root);
    std::thread::Builder::new()
        .name("craft-assistant".into())
        .spawn(move || {
            let result = (|| -> Result<(), String> {
                let shared = SharedStore::new(root.clone());
                let history = history(&shared, &app_id)?;
                if conversation.is_none() {
                    conversation = Some(history.create(&title).map_err(fail)?);
                }
                let Some(mut saved) = conversation else {
                    return Err("could not create the conversation".into());
                };
                saved.messages = request.messages.clone();
                saved.interrupted = true;
                history.save(&saved).map_err(fail)?;
                let _ = events_tx.send(WorkerEvent::Saved(saved.clone()));
                let provider = Providers { http: HttpProvider::new(Arc::new(KeyringStore)), codex };
                let mut host = McpHost::new(call_tx, worker_cancel.clone(), wake.clone(), Duration::from_secs(request.config.request_timeout_secs));
                host.set_limits(&request.config);
                let mut partial = String::new();
                let mut persistence_error = None;
                let outcome = run_turn(request, &provider, &mut host, &worker_cancel, |event| {
                    match &event {
                        TurnEvent::Routed { .. } => partial.clear(),
                        TurnEvent::Text { text, .. } => partial.push_str(text),
                        TurnEvent::Checkpoint { messages } => {
                            partial.clear();
                            if let Err(error) = save_checkpoint(&history, &mut saved, messages, &worker_cancel) {
                                persistence_error.get_or_insert(error);
                            }
                            // Checkpoints are durable recovery state; streamed text remains on screen.
                            return;
                        }
                        _ => {}
                    }
                    let _ = events_tx.send(WorkerEvent::Runtime(event));
                    wake.request_repaint();
                });
                match outcome {
                    Ok(messages) => {
                        saved.messages = messages;
                        saved.interrupted = false;
                    }
                    Err(error) => {
                        if !partial.is_empty() {
                            saved.messages.push(Message::text(MessageRole::Assistant, partial));
                        }
                        saved.messages.push(Message::text(
                            MessageRole::Assistant,
                            format!("Interrupted: {error}. Completed edits may remain; inspect the document before requesting more changes."),
                        ));
                    }
                }
                if let Err(error) = history.save(&saved) {
                    saved.interrupted = true;
                    persistence_error.get_or_insert_with(|| error.to_string());
                }
                // Deliver the latest transcript even when its final disk write failed. Continuing
                // manually must retain known command outcomes and must never replay them.
                let _ = events_tx.send(WorkerEvent::Saved(saved));
                if let Some(error) = persistence_error {
                    return Err(format!("Conversation could not be saved: {error}. The latest transcript is still visible in this window."));
                }
                Ok(())
            })();
            if let Err(message) = result {
                let _ = events_tx.send(WorkerEvent::Runtime(TurnEvent::Failed { message }));
            }
            wake.request_repaint();
        })
        .map_err(fail)?;
    app.assistant.widget.messages = messages;
    app.assistant.widget.activity.clear();
    app.assistant.widget.error = None;
    app.assistant.widget.busy = true;
    app.assistant.widget.status = Some(tl!("Connecting…").into());
    app.assistant.native.turn = Some(Turn { events, calls, cancel, guard, fresh_text: true, pending_activity: Vec::new() });
    Ok(())
}

fn save_checkpoint(history: &HistoryStore, saved: &mut Conversation, messages: &[Message], cancel: &Cancellation) -> Result<(), String> {
    saved.messages = messages.to_vec();
    saved.interrupted = true;
    history.save(saved).map_err(|error| {
        // Runtime checks cancellation before dispatching the next host request.
        cancel.cancel();
        error.to_string()
    })
}

fn settings_action(store: &SharedStore, app: &AppId, requested: Action, cancel: &Cancellation) -> Result<Update, String> {
    cancel.check().map_err(fail)?;
    match requested {
        Action::SaveBinding { role: AssistantRole::Control, .. } | Action::UseSuiteDefault { role: AssistantRole::Control } => {
            Err("GridCraft uses one assistant model. Change the Model selection instead.".into())
        }
        Action::SaveConnection(prepared) => {
            let id = prepared.connection.id.clone();
            save_connection(store, prepared, &KeyringStore).map(|change| Update::ConnectionSaved { change, id })
        }
        Action::DiscoverModels(draft) | Action::TestConnection(draft) => discover(store, draft, None, cancel),
        Action::VerifyModel { connection, model } => discover(store, connection, Some(model), cancel),
        Action::SaveBinding { role, connection, model, app_override, manual_tools } => {
            let connection = ConnectionId::new(connection).map_err(fail)?;
            let model = ModelId::new(model).map_err(fail)?;
            let declaration = role.declaration().map_err(fail)?;
            let stored = store
                .update(|stored| {
                    let selected =
                        stored.connections.iter_mut().find(|c| c.id == connection).ok_or_else(|| AiError::invalid("select a saved connection"))?;
                    if manual_tools {
                        let model_info = tool_model(model.clone())?;
                        selected.models.retain(|m| m.id != model);
                        selected.models.push(model_info);
                    }
                    let binding = Binding { role: declaration.id.clone(), connection: connection.clone(), model: model.clone() };
                    let resolved = craft_ai_core::role::resolve(&declaration, &[std::slice::from_ref(&binding)], &stored.connections)
                        .map_err(|e| AiError::invalid(e.to_string()))?;
                    if !resolved.spec.features.tools {
                        return Err(AiError::unsupported("verify tool support before selecting this model"));
                    }
                    let bindings = if app_override { &mut stored.apps.entry(app.clone()).or_default().overrides } else { &mut stored.defaults };
                    bindings.retain(|b| b.role != binding.role);
                    bindings.push(binding);
                    Ok(())
                })
                .map_err(fail)?;
            Ok(Update::Store(stored))
        }
        Action::UseSuiteDefault { role } => Ok(Update::Store(
            store
                .update(|stored| {
                    if let Some(settings) = stored.apps.get_mut(app) {
                        settings.overrides.retain(|binding| binding.role.as_str() != role.id());
                    }
                    Ok(())
                })
                .map_err(fail)?,
        )),
        Action::DeleteConnection(id) => delete_connection(store, &id, &KeyringStore).map(|change| Update::ConnectionDeleted { change, id }),
        Action::NewChat => Ok(Update::Chat(history(store, app)?.create("New chat").map_err(fail)?)),
        Action::OpenChat(id) => Ok(Update::Chat(history(store, app)?.load(&id).map_err(fail)?)),
        Action::DeleteChat(id) => {
            history(store, app)?.delete(&id).map_err(fail)?;
            Ok(Update::Deleted(id))
        }
        Action::RefreshCodex => refresh_codex(store, cancel),
        _ => Err("this assistant action is not a settings operation".into()),
    }
}

struct ConnectionChange {
    store: StoreFile,
    warning: Option<String>,
}

static NEXT_SECRET: AtomicU64 = AtomicU64::new(1);

// Stage a replacement under a fresh locator. A failed settings write must never destroy the
// credential used by the previously saved connection, even if another Craft app is running.
fn save_connection(store: &SharedStore, mut prepared: Prepared, secrets: &dyn SecretStore) -> Result<ConnectionChange, String> {
    prepared.connection.validate().map_err(fail)?;
    let staged = if let KeyChange::Set(value) = &prepared.key {
        let Auth::ApiKey { secret } = &mut prepared.connection.auth else {
            return Err("Only API connections accept a stored API key.".into());
        };
        *secret = SecretRef::new(format!(
            "craft-key-{}-{}-{}",
            chrono::Utc::now().timestamp_micros(),
            std::process::id(),
            NEXT_SECRET.fetch_add(1, Ordering::Relaxed)
        ))
        .map_err(fail)?;
        secrets.set(secret, value).map_err(|_| {
            "The system credential store could not save the key. The previous connection is unchanged; keep the key in this form and try again."
                .to_string()
        })?;
        Some(secret.clone())
    } else {
        None
    };
    let id = prepared.connection.id.clone();
    let mut previous = None;
    let saved = store.update(|stored| {
        if let Some(current) = stored.connections.iter().find(|connection| connection.id == id) {
            if let Auth::ApiKey { secret } = &current.auth {
                previous = Some(secret.clone());
            }
            if matches!(prepared.key, KeyChange::Keep) && matches!(prepared.connection.auth, Auth::ApiKey { .. }) {
                // An empty key field means keep the latest acknowledged key, not a stale locator.
                prepared.connection.auth = current.auth.clone();
            }
        }
        stored.connections.retain(|connection| connection.id != id);
        stored.connections.push(prepared.connection);
        Ok(())
    });
    match saved {
        Ok(saved) => Ok(ConnectionChange { warning: cleanup_key(store, secrets, previous.as_ref()), store: saved }),
        Err(error) => {
            // A filesystem error may occur after atomic rename. Never delete the replacement
            // credential unless a fresh read proves that no committed connection references it.
            let suffix = if let Some(reference) = staged.as_ref() {
                match store.load() {
                    Ok(current) if current.connections.iter().any(|connection| connection.auth.secret_ref() == Some(reference)) => {
                        return Ok(ConnectionChange {
                            store: current,
                            warning: Some(
                                "The connection was saved, but the settings write reported a filesystem error. The replacement key was retained."
                                    .into(),
                            ),
                        });
                    }
                    Ok(_) if secrets.delete(reference).is_ok() => "",
                    _ => " The replacement key was retained in the credential store because cleanup could not be confirmed.",
                }
            } else {
                ""
            };
            Err(format!("{error}. The previous key was preserved and the form is available to retry.{suffix}"))
        }
    }
}

fn delete_connection(store: &SharedStore, id: &ConnectionId, secrets: &dyn SecretStore) -> Result<ConnectionChange, String> {
    let mut previous = None;
    let saved = store
        .update(|stored| {
            if let Some(Connection { auth: Auth::ApiKey { secret }, .. }) = stored.connections.iter().find(|connection| &connection.id == id) {
                previous = Some(secret.clone());
            }
            stored.connections.retain(|connection| &connection.id != id);
            // Preserve unavailable bindings so deleting a connection can never select a fallback.
            Ok(())
        })
        .map_err(fail)?;
    Ok(ConnectionChange { warning: cleanup_key(store, secrets, previous.as_ref()), store: saved })
}

fn cleanup_key(store: &SharedStore, secrets: &dyn SecretStore, reference: Option<&SecretRef>) -> Option<String> {
    let reference = reference?;
    // Check references while holding the shared settings lock. A key shared by another saved
    // connection remains valid; only an unreferenced API credential may be removed.
    store
        .update(|stored| {
            if !stored.connections.iter().any(|connection| connection.auth.secret_ref() == Some(reference)) {
                secrets.delete(reference)?;
            }
            Ok(())
        })
        .err()
        .map(|_| "Settings were saved, but an unused old API key could not be removed from the system credential store.".into())
}

struct DraftSecrets {
    temporary: Option<(SecretRef, SecretString)>,
}
impl SecretStore for DraftSecrets {
    fn get(&self, reference: &SecretRef) -> Result<SecretString, AiError> {
        if let Some((key, value)) = &self.temporary
            && key == reference
        {
            return Ok(value.clone());
        }
        KeyringStore.get(reference)
    }
    fn set(&self, _: &SecretRef, _: &SecretString) -> Result<(), AiError> {
        Err(AiError::invalid("a connection test never stores credentials"))
    }
    fn delete(&self, _: &SecretRef) -> Result<(), AiError> {
        Err(AiError::invalid("a connection test never deletes credentials"))
    }
}

fn discover(store: &SharedStore, mut draft: ConnectionDraft, verify: Option<String>, cancel: &Cancellation) -> Result<Update, String> {
    let prepared = draft.prepare(&store.load().map_err(fail)?.connections).map_err(fail)?;
    let temporary = match prepared.key {
        KeyChange::Set(value) => prepared.connection.auth.secret_ref().map(|reference| (reference.clone(), value)),
        _ => None,
    };
    let providers = Providers::new(store.root(), Arc::new(DraftSecrets { temporary }));
    let config = store.config().map_err(fail)?;
    let models = if let Some(model) = verify {
        let id = ModelId::new(model).map_err(fail)?;
        let info = craft_ai_native::runtime::verify_model(&providers, &prepared.connection, &id, &config, cancel).map_err(fail)?;
        let mut models = draft.models.clone();
        models.retain(|m| m.id != info.id);
        models.push(info);
        models
    } else {
        providers.models(&prepared.connection, &config, cancel).map_err(fail)?
    };
    let endpoint = draft.endpoint.clone();
    if !draft.set_models(&endpoint, models.clone()) {
        return Err("The connection changed while its models were loading; try again.".into());
    }
    Ok(Update::Models { draft, models })
}

fn codex_connection() -> Result<Connection, String> {
    Ok(Connection {
        id: ConnectionId::new("craft-codex").map_err(fail)?,
        label: "Codex account".into(),
        provider: "codex".into(),
        endpoint: None,
        locality: craft_ai_core::Locality::Remote,
        auth: Auth::Subscription { route: "codex".into(), secret: SecretRef::new("craft-codex-account").map_err(fail)? },
        billing: craft_ai_core::Billing::SubscriptionAllowance,
        models: Vec::new(),
        enabled: true,
        key_header: None,
    })
}

fn refresh_codex(store: &SharedStore, cancel: &Cancellation) -> Result<Update, String> {
    let provider = CodexProvider::new(CodexOptions::in_shared_root(store.root()));
    let config = store.config().map_err(fail)?;
    if !provider.signed_in(&config, cancel).map_err(fail)? {
        return Ok(Update::Notice("Sign into the Craft Codex account first.".into()));
    }
    provider.verify_isolation(&config, cancel).map_err(fail)?;
    let mut connection = codex_connection()?;
    connection.models = provider.models(&connection, &config, cancel).map_err(fail)?;
    save_connection(store, Prepared { connection, key: KeyChange::Keep }, &KeyringStore).map(|change| Update::Store(change.store))
}

fn login(store: &SharedStore, sender: &mpsc::Sender<Result<Update, String>>, cancel: &Cancellation) -> Result<Update, String> {
    let provider = CodexProvider::new(CodexOptions::in_shared_root(store.root()));
    let config = store.config().map_err(fail)?;
    let mut session = provider.start_login(&config, cancel).map_err(fail)?;
    let _ = sender.send(Ok(Update::Login { url: session.authorization_url.clone(), code: session.user_code.clone() }));
    loop {
        cancel.check().map_err(fail)?;
        match session.poll(cancel).map_err(fail)? {
            Some(true) => return refresh_codex(store, cancel),
            Some(false) => return Err("Codex sign-in was not completed".into()),
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

pub(super) fn poll(app: &mut SheetApp, ctx: &egui::Context) {
    poll_job(app, ctx);
    let Some(mut turn) = app.assistant.native.turn.take() else { return };
    while let Ok(pending) = turn.calls.try_recv() {
        host::dispatch(app, ctx, &mut turn.guard, pending);
    }
    let mut disconnected = false;
    loop {
        match turn.events.try_recv() {
            Ok(WorkerEvent::Runtime(event)) => apply_event(app, &mut turn, event),
            Ok(WorkerEvent::Saved(conversation)) => set_chat(app, conversation),
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                disconnected = true;
                break;
            }
        }
    }
    if disconnected {
        app.assistant.widget.busy = false;
    } else {
        app.assistant.native.turn = Some(turn);
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

fn apply_event(app: &mut SheetApp, turn: &mut Turn, event: TurnEvent) {
    let state = &mut app.assistant.widget;
    match event {
        TurnEvent::Routed { connection, model, .. } => {
            let route = format!("{connection} / {model}");
            state.chat_route = route.clone();
            state.control_route = route.clone();
            state.status = Some(route);
            turn.fresh_text = true;
        }
        TurnEvent::Text { text, .. } => {
            if turn.fresh_text {
                state.messages.push(Message::text(MessageRole::Assistant, ""));
                turn.fresh_text = false;
            }
            if let Some(message) = state.messages.last_mut() {
                message.content.push_str(&text);
            }
        }
        TurnEvent::ToolStarted { call } => {
            turn.pending_activity.push((call.id, state.activity.len()));
            state.activity.push(Activity { label: call.name, detail: tl!("Running…").into(), failed: false });
        }
        TurnEvent::ToolFinished { id, result } => {
            if let Some(position) = turn.pending_activity.iter().rposition(|(pending, _)| pending == &id) {
                let (_, index) = turn.pending_activity.remove(position);
                if let Some(activity) = state.activity.get_mut(index) {
                    activity.detail = result.to_string();
                    activity.failed = result.get("isError").and_then(Value::as_bool) == Some(true);
                }
            }
        }
        TurnEvent::Checkpoint { .. } => {}
        TurnEvent::Finished { messages } => {
            state.messages = messages;
            state.status = Some(tl!("Complete").into());
        }
        TurnEvent::Failed { message } => {
            state.error = Some(format!("{message}. {}", tl!("Completed edits remain undoable.")));
            for (_, index) in turn.pending_activity.drain(..) {
                if let Some(activity) = state.activity.get_mut(index) {
                    activity.detail = tl!("Interrupted; inspect the document before requesting more changes.").into();
                    activity.failed = true;
                }
            }
            state.status = None;
        }
    }
}

fn set_chat(app: &mut SheetApp, conversation: Conversation) {
    let state = &mut app.assistant.widget;
    state.messages = conversation.messages.clone();
    state.active_chat = Some(conversation.id.clone());
    state.history.retain(|c| c.id != conversation.id);
    state.history.insert(0, ChatSummary { id: conversation.id.clone(), title: conversation.title.clone(), interrupted: conversation.interrupted });
    state.status = conversation.interrupted.then(|| tl!("Interrupted conversation. Inspect the document before requesting further edits.").into());
    app.assistant.native.conversation = Some(conversation);
}

fn set_store(app: &mut SheetApp, store: StoreFile) {
    if let Ok(id) = app_id() {
        let limits = app.assistant.effective.lines().last().unwrap_or("").to_owned();
        app.assistant.effective = match resolve_choice(&store, &id) {
            Ok(choice) => format!(
                "Provider: {} · Endpoint: {}\n{limits}",
                choice.connection.provider,
                choice.connection.endpoint.as_deref().unwrap_or("Codex managed runtime").split(['?', '#']).next().unwrap_or("")
            ),
            Err(_) => format!("Provider unavailable\n{limits}"),
        };
        let route = resolve_choice(&store, &id)
            .map(|choice| format!("{} / {}", choice.connection.label, choice.model))
            .unwrap_or_else(|error| format!("{}: {error}", tl!("Unavailable")));
        app.assistant.widget.chat_route = route.clone();
        app.assistant.widget.control_route = route;
        app.assistant.widget.settings.codex_status = if store.connections.iter().any(|connection| connection.provider == "codex") {
            tl!("Codex connection saved.").into()
        } else {
            String::new()
        };
        app.assistant.widget.settings.set_store(store, Some(id));
    }
}

fn poll_job(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(receiver) = app.assistant.native.job.take() else { return };
    loop {
        match receiver.try_recv() {
            Ok(Ok(update)) => match update {
                Update::Loaded { root, store, history, config } => {
                    app.assistant.effective = format!(
                        "Context: {} bytes; response: {} bytes; tool rounds: {}. Configure runtime.json in the shared Craft/ai directory.",
                        config.max_context_bytes, config.max_response_bytes, config.max_rounds
                    );
                    if app.assistant.native.root.as_ref() != Some(&root) {
                        app.assistant.native.codex_template = None;
                    }
                    app.assistant.native.root = Some(root);
                    app.assistant.widget.history = history;
                    set_store(app, store);
                }
                Update::Store(store) => {
                    set_store(app, store);
                    app.assistant.widget.settings.status = Some(tl!("Settings saved.").into());
                }
                Update::ConnectionSaved { change, id } => {
                    let draft = change.store.connections.iter().find(|connection| connection.id == id).map(ConnectionDraft::from_connection);
                    set_store(app, change.store);
                    if let Some(draft) = draft {
                        app.assistant.widget.settings.connection = draft;
                    }
                    app.assistant.widget.settings.status = Some(tl!("Connection saved.").into());
                    app.assistant.widget.settings.error = change.warning;
                }
                Update::ConnectionDeleted { change, id } => {
                    set_store(app, change.store);
                    if app.assistant.widget.settings.connection.editing.as_ref() == Some(&id) {
                        app.assistant.widget.settings.connection = ConnectionDraft::default();
                        app.assistant.widget.settings.preset.clear();
                    }
                    app.assistant.widget.settings.status = Some(tl!("Connection deleted.").into());
                    app.assistant.widget.settings.error = change.warning;
                }
                Update::Models { draft, models } => {
                    let current = &mut app.assistant.widget.settings.connection;
                    if current.provider == draft.provider && current.editing == draft.editing && current.set_models(&draft.endpoint, models) {
                        app.assistant.widget.settings.status = Some(tl!("Model list updated. Save the connection to keep it.").into());
                    }
                }
                Update::Chat(chat) => {
                    app.assistant.widget.activity.clear();
                    app.assistant.widget.error = None;
                    set_chat(app, chat);
                }
                Update::Deleted(id) => {
                    app.assistant.widget.history.retain(|c| c.id != id);
                    if app.assistant.widget.active_chat.as_ref() == Some(&id) {
                        app.assistant.widget.active_chat = None;
                        app.assistant.widget.messages.clear();
                        app.assistant.native.conversation = None;
                    }
                }
                Update::Login { url, code } => {
                    app.assistant.widget.settings.codex_status = format!("{} {code}", tl!("Complete sign-in in your browser. Code:"));
                    ctx.open_url(egui::OpenUrl::new_tab(url));
                }
                Update::Notice(message) => app.assistant.widget.settings.status = Some(message),
            },
            Ok(Err(error)) => {
                app.assistant.widget.settings.codex_status.clear();
                app.assistant.widget.error = Some(error.clone());
                app.assistant.widget.settings.error = Some(error);
            }
            Err(mpsc::TryRecvError::Empty) => {
                app.assistant.native.job = Some(receiver);
                ctx.request_repaint_after(Duration::from_millis(100));
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                app.assistant.widget.settings.busy = false;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests;
