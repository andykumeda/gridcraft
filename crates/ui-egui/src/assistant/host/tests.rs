use super::*;
use egui_kittest::kittest::Queryable as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn app() -> SheetApp {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    SheetApp::new(session, crate::Services::default())
}
fn host(sender: mpsc::Sender<Pending>) -> McpHost {
    McpHost::new(sender, Cancellation::default(), egui::Context::default(), Duration::from_secs(5))
}
fn call(app: &mut SheetApp, name: &str, arguments: Value) -> Result<Value, AiError> {
    let (sender, receive) = mpsc::channel();
    let mut guard = Guard::new(app, String::new(), Vec::new());
    let target = guard.target();
    let name = name.to_owned();
    let (done, result) = mpsc::channel();
    std::thread::spawn(move || {
        let mut host = host(sender);
        let _ = done.send(host.call(&target, &ToolCall { id: "one".into(), name, arguments }, &Cancellation::default()));
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(result) = result.try_recv() {
            return result;
        }
        assert!(Instant::now() < deadline, "MCP operation did not finish");
        match receive.recv_timeout(Duration::from_millis(5)) {
            Ok(pending) => dispatch(app, &egui::Context::default(), &mut guard, pending),
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }
}
fn body(result: &Value) -> Value {
    assert_eq!(result["isError"], false, "{result}");
    serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap()
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let root = std::env::temp_dir().join(format!("gridcraft-mcp-parity-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn assistant_catalog_equals_every_external_mcp_name_description_and_schema() {
    let (sender, _) = mpsc::channel();
    let mut host = host(sender);
    let tools = host.tools(&Target { id: "0".into(), revision: 0 }, &Cancellation::default()).unwrap();
    let external = gridcraft_mcp::tool_definitions();
    assert_eq!(tools.len(), external.len());
    for (actual, expected) in tools.iter().zip(&external) {
        assert_eq!(actual.to_mcp(), *expected);
        assert_eq!(actual.name, expected["name"].as_str().unwrap());
        assert_eq!(actual.description, expected["description"].as_str().unwrap());
        assert_eq!(actual.input_schema, expected["inputSchema"]);
    }
    for name in ["execute_command", "list_windows", "new_workbook", "open_workbook", "save_workbook", "screenshot"] {
        assert!(tools.iter().any(|tool| tool.name == name));
    }
}

#[test]
fn formerly_blocked_commands_windows_new_open_and_save_use_live_mcp_dispatch() {
    let mut app = app();
    let temp = Temp::new();
    let destination = temp.0.join("parity.xlsx");
    body(&call(&mut app, "execute_command", json!({"command":"cell.set","params":{"cell":"A1","input":"=2+3"}})).unwrap());
    assert_eq!(body(&call(&mut app, "get_cell", json!({"cell":"A1"})).unwrap())["value"], 5.0);
    body(&call(&mut app, "undo", json!({})).unwrap());
    assert!(body(&call(&mut app, "get_cell", json!({"cell":"A1"})).unwrap())["value"].is_null());
    body(&call(&mut app, "redo", json!({})).unwrap());
    body(&call(&mut app, "save_workbook", json!({"path":destination})).unwrap());
    assert!(destination.is_file());
    let original = app.session.active().unwrap().uid;
    body(&call(&mut app, "set_cell", json!({"cell":"B1","input":"keep dirty workbook"})).unwrap());
    body(&call(&mut app, "new_workbook", json!({})).unwrap());
    assert_ne!(app.session.active().unwrap().uid, original);
    assert!(app.session.documents().iter().any(|doc| doc.uid == original && doc.is_dirty()));
    let listed = body(&call(&mut app, "list_windows", json!({})).unwrap());
    assert!(listed.to_string().contains("Book"));
    body(&call(&mut app, "execute_command", json!({"command":"view.newWindow","params":{}})).unwrap());
    body(&call(&mut app, "open_workbook", json!({"path":destination})).unwrap());
    assert_eq!(body(&call(&mut app, "get_cell", json!({"cell":"A1"})).unwrap())["value"], 5.0);
    assert!(app.session.documents().iter().any(|doc| doc.uid == original && doc.is_dirty()));
    let commands = body(&call(&mut app, "list_commands", json!({"search":"language"})).unwrap());
    assert!(commands.as_array().unwrap().iter().any(|command| command["id"] == "app.language.set"));
}

#[test]
fn explicit_document_sheet_and_range_options_match_the_external_server() {
    let mut app = app();
    body(&call(&mut app, "execute_command", json!({"command":"home.insertSheet","params":{"name":"Other"}})).unwrap());
    body(&call(&mut app, "set_cell", json!({"cell":"Other!A1","input":"cross sheet"})).unwrap());
    let result = call(&mut app, "read_range", json!({"sheet":"Other","formulas":true,"formatted":true})).unwrap();
    assert_eq!(result["isError"], false);
    assert!(result.to_string().contains("cross sheet"));
    let document = app.session.active().unwrap().uid;
    body(&call(&mut app, "new_workbook", json!({})).unwrap());
    let result = call(&mut app, "get_cell", json!({"cell":"Other!A1","doc":document})).unwrap();
    assert_eq!(body(&result)["value"], "cross sheet");
}

#[test]
fn tool_results_equal_external_server_results_without_repacking() {
    struct Direct {
        app: SheetApp,
        ctx: egui::Context,
    }
    impl Backend for Direct {
        fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
            match control::handle(&mut self.app, &self.ctx, method, &params) {
                control::Outcome::Done(reply) if reply["ok"] == true => Ok(reply["result"].clone()),
                control::Outcome::Done(reply) => Err(reply["error"].as_str().unwrap_or("unknown error").into()),
                control::Outcome::Screenshot { .. } => Err("deferred capture is tested separately".into()),
            }
        }
        fn has_ui(&self) -> bool {
            true
        }
        fn describe(&self) -> String {
            "GridCraft test UI".into()
        }
    }
    let mut external = Server::new(Box::new(Direct { app: app(), ctx: egui::Context::default() }));
    let mut embedded = app();
    for (name, args) in [("get_cell", json!({"cell":"A1"})), ("execute_command", json!({"command":"not.a.command"}))] {
        let expected = external.handle(json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}})).unwrap();
        assert_eq!(call(&mut embedded, name, args).unwrap(), expected["result"]);
    }
}

#[test]
fn authoritative_errors_and_full_result_envelopes_are_preserved() {
    let mut app = app();
    let failed = call(&mut app, "execute_command", json!({"command":"not.a.command"})).unwrap();
    assert_eq!(failed["isError"], true);
    assert!(failed["content"][0]["text"].as_str().unwrap().contains("not.a.command"));
    assert!(call(&mut app, "save_workbook", json!({"path":42})).is_err());
    body(&call(&mut app, "execute_command", json!({"command":"home.bold","params":{"range":"A1"}})).unwrap());
    let result = call(&mut app, "get_cell", json!({"cell":"A1"})).unwrap();
    assert!(body(&result)["style"]["font"]["bold"].as_bool().unwrap_or(false));
}

#[test]
fn canceled_or_expired_control_requests_do_not_run_late() {
    let mut app = app();
    let ctx = egui::Context::default();
    let mut guard = Guard::new(&app, String::new(), vec![]);
    for expired in [false, true] {
        let (request, receive) =
            control::ControlRequest::new("engine.execute", json!({"command":"cell.set","params":{"cell":"A1","input":"must not run"}}));
        let cancel = Cancellation::default();
        if !expired {
            cancel.cancel();
        }
        dispatch(
            &mut app,
            &ctx,
            &mut guard,
            Pending {
                request,
                cancel,
                expires: if expired { Instant::now() - Duration::from_secs(1) } else { Instant::now() + Duration::from_secs(1) },
            },
        );
        assert!(receive.try_recv().is_err());
    }
    assert!(app.session.execute("cell.get", json!({"cell":"A1"})).unwrap()["value"].is_null());
}

#[test]
fn screenshot_uses_the_same_async_gui_capture_and_png_writer_as_external_mcp() {
    let temp = Temp::new();
    let path = temp.0.join("capture.png");
    let mut app = app();
    app.use_prepared_fonts();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .with_context_setup(|ctx| {
            ctx.set_fonts(crate::theme::pinned_font_definitions());
            crate::theme::apply(ctx, false);
        })
        .renderer(egui_kittest::LazyRenderer::Uninitialized { textures_delta: Default::default(), builder: None })
        .build_ui_state(
            |ui, app: &mut SheetApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
    harness.run_steps(3);
    assert!(harness.query_by_label("Name Box").is_some());
    let ctx = harness.ctx.clone();
    let mut guard = Guard::new(harness.state(), String::new(), vec![]);
    let target = guard.target();
    let (sender, pending) = mpsc::channel();
    let (done, result) = mpsc::channel();
    let destination = path.clone();
    std::thread::spawn(move || {
        let mut host = host(sender);
        let _ = done.send(host.call(
            &target,
            &ToolCall { id: "capture".into(), name: "screenshot".into(), arguments: json!({"path":destination}) },
            &Cancellation::default(),
        ));
    });
    dispatch(harness.state_mut(), &ctx, &mut guard, pending.recv_timeout(Duration::from_secs(2)).unwrap());
    assert!(harness.state().shots.in_flight());
    assert!(result.try_recv().is_err());
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut completed = false;
    loop {
        harness.step();
        let callbacks: Vec<_> = harness
            .output()
            .viewport_output
            .values()
            .flat_map(|viewport| &viewport.commands)
            .filter_map(|command| match command {
                egui::ViewportCommand::Screenshot(callback) => Some(callback.clone()),
                _ => None,
            })
            .collect();
        for callback in callbacks {
            callback.complete(std::sync::Arc::new(egui::ColorImage::filled([900, 700], egui::Color32::LIGHT_BLUE)));
            completed = true;
        }
        if let Ok(reply) = result.try_recv() {
            assert_eq!(reply.unwrap()["isError"], false);
            break;
        }
        assert!(Instant::now() < deadline, "screenshot did not complete");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(completed);
    assert!(path.is_file());
    assert_eq!(image::open(path).unwrap().width(), 900);
}
