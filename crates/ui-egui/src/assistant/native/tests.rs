use super::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("gridcraft-native-assistant-{}-{}", std::process::id(), NEXT_SECRET.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> SharedStore {
        SharedStore::new(self.0.clone())
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct Secrets {
    values: Mutex<BTreeMap<SecretRef, SecretString>>,
    fail_set: bool,
    fail_delete: bool,
}
impl SecretStore for Secrets {
    fn get(&self, reference: &SecretRef) -> Result<SecretString, AiError> {
        self.values.lock().unwrap().get(reference).cloned().ok_or_else(|| AiError::invalid("missing test credential"))
    }
    fn set(&self, reference: &SecretRef, value: &SecretString) -> Result<(), AiError> {
        if self.fail_set {
            return Err(AiError::invalid("untrusted error with synthetic-private-key"));
        }
        self.values.lock().unwrap().insert(reference.clone(), value.clone());
        Ok(())
    }
    fn delete(&self, reference: &SecretRef) -> Result<(), AiError> {
        if self.fail_delete {
            return Err(AiError::invalid("untrusted error with synthetic-private-key"));
        }
        self.values.lock().unwrap().remove(reference);
        Ok(())
    }
}
fn draft() -> ConnectionDraft {
    let mut draft = ConnectionDraft::default();
    draft.label = "Synthetic endpoint".into();
    draft.endpoint = "https://example.com/v1".into();
    draft.key = "synthetic-private-key".into();
    draft
}
fn save(store: &SharedStore, secrets: &Secrets) -> Connection {
    save_connection(store, draft().prepare(&[]).unwrap(), secrets).unwrap().store.connections.remove(0)
}
fn app() -> SheetApp {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    SheetApp::new(session, crate::Services::default())
}

#[test]
fn replaced_keys_are_staged_then_old_keys_removed_without_plaintext_settings() {
    let temp = Temp::new();
    let store = temp.store();
    let secrets = Secrets::default();
    let original = save(&store, &secrets);
    let mut edit = ConnectionDraft::from_connection(&original);
    edit.key = "synthetic-replacement-key".into();
    let changed = save_connection(&store, edit.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).unwrap();
    let replacement = changed.store.connections.first().unwrap();
    assert_ne!(replacement.auth.secret_ref(), original.auth.secret_ref());
    assert!(secrets.get(original.auth.secret_ref().unwrap()).is_err());
    assert_eq!(secrets.get(replacement.auth.secret_ref().unwrap()).unwrap().expose(), "synthetic-replacement-key");
    let contents = std::fs::read_to_string(temp.0.join("connections.json")).unwrap();
    assert!(!contents.contains("synthetic-private-key"));
    assert!(!contents.contains("synthetic-replacement-key"));
    assert!(changed.warning.is_none());
}

#[test]
fn credential_failure_preserves_saved_state_and_redacts_untrusted_error() {
    let temp = Temp::new();
    let store = temp.store();
    let mut secrets = Secrets::default();
    let original = save(&store, &secrets);
    secrets.fail_set = true;
    let error = save_connection(&store, draft().prepare(std::slice::from_ref(&original)).unwrap(), &secrets).err().unwrap();
    assert!(!error.contains("synthetic-private-key"));
    assert_eq!(store.load().unwrap().connections, vec![original.clone()]);
    assert_eq!(secrets.get(original.auth.secret_ref().unwrap()).unwrap().expose(), "synthetic-private-key");
}

#[test]
fn failed_settings_write_never_destroys_the_previous_credential() {
    let temp = Temp::new();
    let store = temp.store();
    let secrets = Secrets::default();
    let original = save(&store, &secrets);
    std::fs::write(temp.0.join("connections.json"), "corrupt synthetic fixture").unwrap();
    let mut edit = ConnectionDraft::from_connection(&original);
    edit.key = "synthetic-new-key".into();
    let error = save_connection(&store, edit.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).err().unwrap();
    assert!(!error.contains("synthetic-new-key"));
    assert_eq!(secrets.get(original.auth.secret_ref().unwrap()).unwrap().expose(), "synthetic-private-key");
}

#[test]
fn clear_and_delete_remove_only_unreferenced_keys() {
    let temp = Temp::new();
    let store = temp.store();
    let secrets = Secrets::default();
    let original = save(&store, &secrets);
    let mut edit = ConnectionDraft::from_connection(&original);
    edit.uses_key = false;
    let cleared = save_connection(&store, edit.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).unwrap();
    assert!(cleared.store.connections.first().unwrap().auth.secret_ref().is_none());
    assert!(secrets.get(original.auth.secret_ref().unwrap()).is_err());
    let first = save(&store, &secrets);
    let mut second = first.clone();
    second.id = ConnectionId::new("second").unwrap();
    store
        .update(|stored| {
            stored.connections.push(second.clone());
            Ok(())
        })
        .unwrap();
    delete_connection(&store, &first.id, &secrets).unwrap();
    assert!(secrets.get(first.auth.secret_ref().unwrap()).is_ok());
    delete_connection(&store, &second.id, &secrets).unwrap();
    assert!(secrets.get(first.auth.secret_ref().unwrap()).is_err());
}

#[test]
fn cleanup_failure_is_a_warning_after_the_connection_was_saved() {
    let temp = Temp::new();
    let store = temp.store();
    let mut secrets = Secrets::default();
    let original = save(&store, &secrets);
    secrets.fail_delete = true;
    let mut edit = ConnectionDraft::from_connection(&original);
    edit.uses_key = false;
    let result = save_connection(&store, edit.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).unwrap();
    assert!(result.store.connections.first().unwrap().auth.secret_ref().is_none());
    let warning = result.warning.unwrap();
    assert!(!warning.contains("synthetic-private-key"));
    assert!(warning.contains("saved"));
}

#[test]
fn stale_empty_key_edit_keeps_the_latest_saved_credential() {
    let temp = Temp::new();
    let store = temp.store();
    let secrets = Secrets::default();
    let original = save(&store, &secrets);
    let stale = ConnectionDraft::from_connection(&original);
    let mut replacement = stale.clone();
    replacement.key = "new-synthetic-key".into();
    let saved = save_connection(&store, replacement.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).unwrap();
    let current = saved.store.connections.first().unwrap().auth.secret_ref().unwrap();
    let updated = save_connection(&store, stale.prepare(std::slice::from_ref(&original)).unwrap(), &secrets).unwrap();
    assert_eq!(updated.store.connections.first().unwrap().auth.secret_ref(), Some(current));
    assert_eq!(secrets.get(current).unwrap().expose(), "new-synthetic-key");
}

#[test]
fn checkpoints_persist_completed_commands_and_cancel_on_write_failure() {
    let temp = Temp::new();
    let history = history(&temp.store(), &app_id().unwrap()).unwrap();
    let mut saved = history.create("Synthetic history").unwrap();
    let mut call = Message::text(MessageRole::Assistant, "");
    call.tool_calls.push(craft_ai_core::assistant::ToolCall {
        id: "delegation".into(),
        name: "write_range".into(),
        arguments: json!({"task":"edit"}),
    });
    let messages = vec![
        Message::text(MessageRole::User, "Edit"),
        call,
        Message::tool("delegation", &json!({"activity":[{"id":"edit-1","outcome":"completed"}]})),
    ];
    let cancel = Cancellation::new();
    save_checkpoint(&history, &mut saved, &messages, &cancel).unwrap();
    let recovered = history.load(&saved.id).unwrap();
    assert!(recovered.interrupted);
    assert_eq!(recovered.messages, messages);
    assert!(!cancel.is_cancelled());
    let file = temp.0.join("chats/gridcraft").join(format!("{}.json", saved.id));
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    assert!(save_checkpoint(&history, &mut saved, &messages, &cancel).is_err());
    assert!(cancel.is_cancelled());
    assert_eq!(saved.messages, messages);
}

#[test]
fn save_acknowledgment_keeps_connection_identity_and_failed_send_keeps_draft() {
    let mut app = app();
    app.assistant.enabled = true;
    app.assistant.native.initialized = true;
    assert!(action(&mut app, Action::Send("keep this unsent prompt".into())).is_err());
    assert_eq!(app.assistant.widget.draft, "keep this unsent prompt");
    let connection = draft().prepare(&[]).unwrap().connection;
    app.assistant.widget.settings.connection = draft();
    let (sender, receiver) = mpsc::channel();
    sender
        .send(Ok(Update::ConnectionSaved {
            change: ConnectionChange { store: StoreFile { connections: vec![connection.clone()], ..Default::default() }, warning: None },
            id: connection.id.clone(),
        }))
        .unwrap();
    drop(sender);
    app.assistant.native.job = Some(receiver);
    poll_job(&mut app, &egui::Context::default());
    assert_eq!(app.assistant.widget.settings.connection.editing, Some(connection.id));
    assert!(app.assistant.widget.settings.connection.key.is_empty());
}

#[test]
fn nested_tool_results_update_their_own_activity_and_removed_routes_are_visible() {
    let mut app = app();
    let mut turn = Turn {
        events: mpsc::channel().1,
        calls: mpsc::channel().1,
        cancel: Cancellation::new(),
        guard: Guard::new(&app, "test".into(), vec![]),
        fresh_text: true,
        pending_activity: Vec::new(),
    };
    for name in ["write_range", "set_cell"] {
        apply_event(
            &mut app,
            &mut turn,
            TurnEvent::ToolStarted { call: craft_ai_core::assistant::ToolCall { id: "same-id".into(), name: name.into(), arguments: json!({}) } },
        );
    }
    apply_event(&mut app, &mut turn, TurnEvent::ToolFinished { id: "same-id".into(), result: json!({"edit":"done"}) });
    apply_event(&mut app, &mut turn, TurnEvent::ToolFinished { id: "same-id".into(), result: json!({"delegation":"done"}) });
    assert!(app.assistant.widget.activity.first().unwrap().detail.contains("delegation"));
    assert!(app.assistant.widget.activity.get(1).unwrap().detail.contains("edit"));
    app.assistant.widget.chat_route = "stale provider".into();
    app.assistant.effective = "Provider: stale-provider · Endpoint: https://example.com/v1\nContext: synthetic limits".into();
    set_store(&mut app, StoreFile::default());
    assert!(!app.assistant.widget.chat_route.contains("stale provider"));
    assert!(app.assistant.widget.chat_route.contains("Unavailable"));
    assert!(!app.assistant.effective.contains("stale-provider"));
    assert!(app.assistant.effective.contains("Context: synthetic limits"));
}

fn model_store(shared: &SharedStore, legacy_control: bool) -> StoreFile {
    let connection = save(shared, &Secrets::default());
    let selected = ModelId::new("selected-model").unwrap();
    let legacy = ModelId::new("legacy-control-model").unwrap();
    shared
        .update(|stored| {
            stored.connections.iter_mut().find(|item| item.id == connection.id).unwrap().models =
                vec![tool_model(selected.clone()).unwrap(), tool_model(legacy.clone()).unwrap()];
            let mut bindings =
                vec![Binding { role: AssistantRole::Chat.declaration().unwrap().id, connection: connection.id.clone(), model: selected }];
            if legacy_control {
                bindings.push(Binding { role: AssistantRole::Control.declaration().unwrap().id, connection: connection.id.clone(), model: legacy });
            }
            stored.apps.entry(app_id().unwrap()).or_default().overrides = bindings;
            Ok(())
        })
        .unwrap()
}

#[test]
fn unified_model_request_uses_one_choice_and_preserves_existing_history() {
    for legacy_control in [false, true] {
        let temporary = Temp::new();
        let shared = temporary.store();
        let stored = model_store(&shared, legacy_control);
        let history = history(&shared, &app_id().unwrap()).unwrap();
        let mut saved = history.create("Existing conversation").unwrap();
        saved.messages = vec![Message::text(MessageRole::User, "Keep this existing conversation.")];
        history.save(&saved).unwrap();
        let mut app = app();
        enable_without_profile(&mut app);
        set_chat(&mut app, saved.clone());
        set_store(&mut app, stored.clone());
        let mut messages = saved.messages.clone();
        messages.push(Message::text(MessageRole::User, "Update this workbook."));
        let request = turn_request(
            &shared,
            app_id().unwrap(),
            messages.clone(),
            Target { id: app.session.active().unwrap().uid.to_string(), revision: app.session.active().unwrap().revision },
        )
        .unwrap();
        assert_eq!(request.chat.model.as_str(), "selected-model");
        assert_eq!(request.chat.model, request.control.model);
        assert_eq!(request.chat.connection, request.control.connection);
        assert_eq!(request.messages, messages);
        assert!(app.assistant.widget.chat_route.ends_with(" / selected-model"));
        assert_eq!(app.assistant.widget.chat_route, app.assistant.widget.control_route);
        let status = crate::assistant::command(&mut app, "ui.assistant.status", &json!({})).unwrap().unwrap();
        assert_eq!(status["model"], status["chatModel"]);
        assert_eq!(status["model"], status["controlModel"]);
        assert_eq!(shared.load().unwrap(), stored, "legacy bindings must not be deleted or rewritten");
        assert_eq!(history.load(&saved.id).unwrap().messages, saved.messages);
        assert_eq!(app.assistant.widget.messages, saved.messages);
        assert_eq!(app.assistant.native.conversation.as_ref().unwrap().id, saved.id);
    }
}

#[test]
fn clearing_the_unified_choice_does_not_fall_back_to_legacy_control() {
    let temporary = Temp::new();
    let shared = temporary.store();
    model_store(&shared, true);
    let Ok(Update::Store(stored)) =
        settings_action(&shared, &app_id().unwrap(), Action::UseSuiteDefault { role: AssistantRole::Chat }, &Cancellation::new())
    else {
        panic!("the canonical app override should be removable");
    };
    assert!(stored.apps.get(&app_id().unwrap()).unwrap().overrides.iter().any(|binding| binding.role.as_str() == AssistantRole::Control.id()));
    let mut app = app();
    enable_without_profile(&mut app);
    set_store(&mut app, stored);
    assert!(app.assistant.widget.chat_route.contains("Unavailable"));
    assert_eq!(app.assistant.widget.chat_route, app.assistant.widget.control_route);
    assert!(turn_request(&shared, app_id().unwrap(), vec![], Target { id: "synthetic".into(), revision: 0 }).is_err());
}

#[test]
fn obsolete_control_binding_actions_leave_saved_choices_unchanged() {
    let temporary = Temp::new();
    let shared = temporary.store();
    let stored = model_store(&shared, true);
    for requested in [
        Action::SaveBinding {
            role: AssistantRole::Control,
            connection: "unused-connection".into(),
            model: "unused-model".into(),
            app_override: true,
            manual_tools: false,
        },
        Action::UseSuiteDefault { role: AssistantRole::Control },
    ] {
        let Err(error) = settings_action(&shared, &app_id().unwrap(), requested, &Cancellation::new()) else {
            panic!("obsolete split-model actions must be refused");
        };
        assert!(error.contains("one assistant model"));
        assert_eq!(shared.load().unwrap(), stored);
    }
}

#[test]
fn gridcraft_conversations_and_model_overrides_are_isolated_from_wordcraft() {
    let temporary = Temp::new();
    let shared = temporary.store();
    let wordcraft = AppId::new("wordcraft").unwrap();
    let other_history = history(&shared, &wordcraft).unwrap();
    let mut original = other_history.create("Other app conversation").unwrap();
    original.messages.push(Message::text(MessageRole::User, "Synthetic other app content"));
    other_history.save(&original).unwrap();
    let gridcraft_history = history(&shared, &app_id().unwrap()).unwrap();
    assert!(gridcraft_history.list().unwrap().is_empty());
    assert!(gridcraft_history.load(&original.id).is_err());
    gridcraft_history.create("GridCraft conversation").unwrap();
    model_store(&shared, false);
    assert!(!shared.load().unwrap().apps.contains_key(&wordcraft));
    assert_eq!(other_history.load(&original.id).unwrap().messages, original.messages);
    assert_eq!(other_history.list().unwrap().len(), 1);
    assert_eq!(gridcraft_history.list().unwrap().len(), 1);
}

#[test]
fn stopped_turn_and_settings_jobs_are_cancelled_without_replaying_edits() {
    let mut app = app();
    enable_without_profile(&mut app);
    let cancellation = Cancellation::new();
    let settings_cancellation = Cancellation::new();
    app.assistant.native.job_cancel = settings_cancellation.clone();
    app.assistant.native.turn = Some(Turn {
        events: mpsc::channel().1,
        calls: mpsc::channel().1,
        cancel: cancellation.clone(),
        guard: Guard::new(&app, "synthetic".into(), vec![]),
        fresh_text: true,
        pending_activity: Vec::new(),
    });
    app.run("cell.set", json!({"cell":"A1","input":"retained edit"})).unwrap();
    action(&mut app, Action::Stop).unwrap();
    assert!(cancellation.is_cancelled());
    assert!(settings_cancellation.is_cancelled());
    assert_eq!(app.session.execute("cell.get", json!({"cell":"A1"})).unwrap()["value"], "retained edit");
    app.run("edit.undo", json!({})).unwrap();
    assert!(app.session.execute("cell.get", json!({"cell":"A1"})).unwrap()["value"].is_null());
}

#[test]
fn interrupted_chat_reopens_as_idle_and_never_dispatches_a_tool() {
    let temporary = Temp::new();
    let shared = temporary.store();
    let history = history(&shared, &app_id().unwrap()).unwrap();
    let mut saved = history.create("Interrupted synthetic edit").unwrap();
    let messages = vec![Message::text(MessageRole::User, "Write A1")];
    save_checkpoint(&history, &mut saved, &messages, &Cancellation::new()).unwrap();
    let Ok(Update::Chat(recovered)) = settings_action(&shared, &app_id().unwrap(), Action::OpenChat(saved.id.clone()), &Cancellation::new()) else {
        panic!("loading the saved synthetic chat must succeed");
    };
    let mut app = app();
    enable_without_profile(&mut app);
    set_chat(&mut app, recovered);
    assert!(!app.assistant.widget.busy);
    assert!(app.assistant.native.turn.is_none());
    assert!(app.assistant.widget.status.as_ref().unwrap().contains("Interrupted"));
    assert_eq!(app.assistant.widget.messages, messages);
    assert!(app.session.execute("cell.get", json!({"cell":"A1"})).unwrap()["value"].is_null());
}

#[test]
fn codex_session_template_resets_for_authentication_and_reload_only() {
    let temporary = Temp::new();
    let mut native = Native::default();
    for reset in [Action::ReloadSettings, Action::BeginCodexSignIn] {
        drop(native.codex_for_turn(&temporary.0));
        let template = Arc::clone(&native.codex_template.as_ref().unwrap().1);
        native.invalidate_codex_on(&Action::NewChat);
        native.cancel();
        assert!(Arc::ptr_eq(&template, &native.codex_template.as_ref().unwrap().1));
        native.invalidate_codex_on(&reset);
        assert!(native.codex_template.is_none());
    }
}

#[test]
fn selected_codex_and_ollama_routes_reach_the_provider_without_fallback() {
    struct Provider {
        calls: Mutex<Vec<(String, String, Option<String>)>>,
        fail: bool,
    }
    impl ChatProvider for Provider {
        fn complete(&self, request: &ModelRequest<'_>, _: &Cancellation, _: &mut dyn FnMut(&str)) -> Result<Message, AiError> {
            self.calls.lock().unwrap().push((
                request.connection.provider.clone(),
                request.model.as_str().into(),
                request.connection.endpoint.clone(),
            ));
            assert!(request.messages.iter().all(|message| !message.content.contains("synthetic workbook sentinel")));
            if self.fail {
                return Err(AiError::invalid("synthetic selected-provider failure"));
            }
            Ok(Message::text(MessageRole::Assistant, "Synthetic response"))
        }
        fn models(&self, _: &Connection, _: &RuntimeConfig, _: &Cancellation) -> Result<Vec<ModelInfo>, AiError> {
            Ok(vec![])
        }
    }
    struct Host;
    impl craft_ai_native::ToolHost for Host {
        fn tools(&mut self, _: &Target, _: &Cancellation) -> Result<Vec<craft_ai_core::assistant::ToolDefinition>, AiError> {
            Ok(vec![craft_ai_core::assistant::ToolDefinition {
                name: "inspect_workbook".into(),
                description: "Synthetic read tool; general chat must not call it".into(),
                input_schema: json!({"type":"object","properties":{}}),
            }])
        }
        fn call(&mut self, _: &Target, _: &craft_ai_core::assistant::ToolCall, _: &Cancellation) -> Result<Value, AiError> {
            Err(AiError::invalid("general chat must not read the workbook"))
        }
    }
    for selected in ["codex", "ollama"] {
        for fail in [false, true] {
            let temporary = Temp::new();
            let shared = temporary.store();
            model_store(&shared, false);
            shared
                .update(|stored| {
                    let connection = stored.connections.first_mut().unwrap();
                    if selected == "codex" {
                        let mut codex = codex_connection().unwrap();
                        codex.id = connection.id.clone();
                        codex.models = connection.models.clone();
                        *connection = codex;
                    } else {
                        connection.provider = "ollama".into();
                        connection.endpoint = Some("http://127.0.0.1:1/v1".into());
                        connection.auth = Auth::None;
                        connection.locality = craft_ai_core::Locality::Local;
                    }
                    Ok(())
                })
                .unwrap();
            let request = turn_request(
                &shared,
                app_id().unwrap(),
                vec![Message::text(MessageRole::User, "Answer a general question")],
                Target { id: "synthetic-target".into(), revision: 0 },
            )
            .unwrap();
            let provider = Provider { calls: Mutex::new(Vec::new()), fail };
            let outcome = run_turn(request, &provider, &mut Host, &Cancellation::new(), |_| {});
            assert_eq!(outcome.is_err(), fail, "{selected}: {outcome:?}");
            let calls = provider.calls.lock().unwrap();
            assert_eq!(calls.len(), 1, "a failed selected provider is never retried or replaced");
            assert_eq!(calls[0].0, selected);
            assert_eq!(calls[0].1, "selected-model");
            if selected == "ollama" {
                assert_eq!(calls[0].2.as_deref(), Some("http://127.0.0.1:1/v1"));
            }
        }
    }
}

#[test]
fn unavailable_file_grants_are_rejected_without_starting_jobs_or_accessing_paths() {
    let mut app = app();
    enable_without_profile(&mut app);
    for request in
        [Action::ChooseFiles, Action::AddFile("/tmp/synthetic-never-opened".into()), Action::RemoveFile("/tmp/synthetic-never-opened".into())]
    {
        let error = action(&mut app, request).unwrap_err();
        assert!(error.contains("file access is unavailable"));
        assert!(app.assistant.widget.authorized_files.is_empty());
        assert!(app.assistant.native.job.is_none());
        assert!(app.assistant.native.turn.is_none());
    }
}
