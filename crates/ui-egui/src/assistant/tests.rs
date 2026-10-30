use super::*;
use egui_kittest::kittest::Queryable as _;

fn app() -> SheetApp {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, crate::Services::default());
    native::enable_without_profile(&mut app);
    app.use_prepared_fonts();
    app
}

#[test]
fn assistant_commands_are_discoverable_and_status_does_not_include_transcripts() {
    let mut app = app();
    app.assistant
        .widget
        .messages
        .push(craft_ai_core::assistant::Message::text(craft_ai_core::assistant::MessageRole::User, "synthetic private workbook text"));
    let commands = commands();
    for id in ["ui.assistant", "ui.assistant.send", "ui.assistant.stop", "ui.assistant.status", "ui.assistant.newChat"] {
        assert!(commands.iter().any(|entry| entry["id"] == id));
    }
    assert_eq!(command(&mut app, "ui.assistant", &json!({"value":true})).unwrap().unwrap()["open"], true);
    assert_eq!(command(&mut app, "ui.assistant", &json!({})).unwrap().unwrap()["open"], false);
    let status = command(&mut app, "ui.assistant.status", &json!({})).unwrap().unwrap();
    assert!(!status.to_string().contains("synthetic private workbook text"));
    assert!(command(&mut app, "ui.assistant.send", &json!({"text":"  "})).unwrap().is_err());
    assert!(command(&mut app, "ui.assistant.send", &json!({"text":"read workbook","files":["/tmp/synthetic"]})).unwrap().is_err());
}

#[test]
fn toolbar_and_view_controls_open_and_close_the_native_panel() {
    let mut app = app();
    app.ui.ribbon_tab = "View".into();
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1300.0, 850.0))
        .with_context_setup(|ctx| {
            ctx.set_fonts(crate::theme::pinned_font_definitions());
            crate::theme::apply(ctx, false);
        })
        .build_ui_state(|ui, app: &mut SheetApp| app.ui(ui), app);
    h.run_steps(3);
    let controls = h.query_all_by_label("Assistant").collect::<Vec<_>>();
    assert_eq!(controls.len(), 2, "the toolbar and View tab each expose Assistant");
    controls[0].click();
    h.run_steps(3);
    assert!(h.state().assistant.open);
    assert!(h.query_by_label("New chat").is_some());
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(!h.state().assistant.open);
}

#[test]
fn settings_render_without_a_workbook_and_keep_advanced_controls_collapsed() {
    let mut app = SheetApp::new(gridcraft_engine::Session::new(), crate::Services::default());
    native::enable_without_profile(&mut app);
    let mut connection = craft_ai_core::ConnectionDraft::default();
    connection.label = "Synthetic local connection".into();
    connection.endpoint = "http://127.0.0.1:1/v1".into();
    connection.uses_key = false;
    let connection = connection.prepare(&[]).unwrap().connection;
    app.assistant.widget.settings.set_store(
        craft_ai_core::StoreFile { connections: vec![connection], ..Default::default() },
        Some(craft_ai_core::AppId::new("gridcraft").unwrap()),
    );
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(950.0, 850.0)).build_ui_state(|ui, app: &mut SheetApp| settings(app, ui), app);
    h.run_steps(3);
    assert!(h.state().session.active().is_none());
    assert!(h.query_by_label("Assistant settings").is_some());
    assert!(h.query_by_label("Assistant model").is_some());
    assert!(h.query_by_label("Advanced model options").is_some());
    assert!(h.query_by_label("General chat").is_none());
    assert!(h.query_by_label("App control").is_none());
    assert!(h.query_by_label("Use suite default").is_none(), "advanced model options start collapsed");
}
