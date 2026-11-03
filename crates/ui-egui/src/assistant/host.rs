//! The assistant uses the external MCP server and the same live-app control dispatcher.
#![allow(clippy::disallowed_methods)] // This module and its clocks compile only off WASM.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use craft_ai_core::assistant::{Target, ToolCall, ToolDefinition};
use craft_ai_core::{AiError, ErrorKind};
use craft_ai_native::{Cancellation, ToolHost};
use gridcraft_mcp::{Backend, Server};
use serde_json::{Value, json};

use crate::{SheetApp, control};

pub(super) struct Pending {
    request: control::ControlRequest,
    cancel: Cancellation,
    expires: Instant,
}

/// Stable identity for the transport's turn envelope; MCP arguments own document/window targeting.
pub(super) struct Guard {
    target: Target,
}
impl Guard {
    pub(super) fn new(app: &SheetApp, _prompt: String, _files: Vec<String>) -> Self {
        Self {
            target: Target {
                id: app.session.active().map_or(0, |doc| doc.uid).to_string(),
                revision: app.session.active().map_or(0, |doc| doc.revision),
            },
        }
    }
    pub(super) fn target(&self) -> Target {
        self.target.clone()
    }
}

struct AppBackend {
    sender: mpsc::Sender<Pending>,
    cancel: Cancellation,
    wake: egui::Context,
    timeout: Duration,
}
impl Backend for AppBackend {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.cancel.check().map_err(|error| error.to_string())?;
        let expires = Instant::now().checked_add(self.timeout).ok_or("assistant timeout is too large")?;
        let (request, receive) = control::ControlRequest::new(method, params);
        let state = request.state.clone();
        self.sender.send(Pending { request, cancel: self.cancel.clone(), expires }).map_err(|_| {
            self.cancel.cancel();
            "the app stopped answering; the operation will not be replayed".to_owned()
        })?;
        self.wake.request_repaint();
        loop {
            if self.cancel.is_cancelled() {
                control::ControlRequest::abandon(&state);
                return Err("assistant operation cancelled; inspect completed actions before continuing".into());
            }
            match receive.recv_timeout(Duration::from_millis(25)) {
                Ok(response) => {
                    return if response.get("ok").and_then(Value::as_bool) == Some(true) {
                        Ok(response.get("result").cloned().unwrap_or(Value::Null))
                    } else {
                        Err(response.get("error").and_then(Value::as_str).unwrap_or("unknown error").to_owned())
                    };
                }
                Err(mpsc::RecvTimeoutError::Timeout) if Instant::now() < expires => {}
                _ => {
                    control::ControlRequest::abandon(&state);
                    self.cancel.cancel();
                    return Err("the app response is unavailable; the operation outcome is unknown and will not be replayed".into());
                }
            }
        }
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn describe(&self) -> String {
        "connected to the GridCraft app".into()
    }
}

pub(super) struct McpHost {
    server: Server,
    cancel: Cancellation,
}
impl McpHost {
    pub(super) fn new(sender: mpsc::Sender<Pending>, cancel: Cancellation, wake: egui::Context, timeout: Duration) -> Self {
        let server = Server::new(Box::new(AppBackend { sender, cancel: cancel.clone(), wake, timeout }));
        Self { server, cancel }
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, AiError> {
        let response = self
            .server
            .handle(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .ok_or_else(|| AiError::invalid("the MCP server did not return a response"))?;
        if let Some(error) = response.get("error") {
            return Err(AiError::invalid(error.get("message").and_then(Value::as_str).unwrap_or("MCP request failed")));
        }
        response.get("result").cloned().ok_or_else(|| AiError::invalid("the MCP server returned an invalid response"))
    }
}
impl ToolHost for McpHost {
    fn tools(&mut self, _target: &Target, cancel: &Cancellation) -> Result<Vec<ToolDefinition>, AiError> {
        cancel.check()?;
        let listed = self.request("tools/list", json!({}))?;
        listed
            .get("tools")
            .and_then(Value::as_array)
            .ok_or_else(|| AiError::invalid("invalid MCP tool catalog"))?
            .iter()
            .map(ToolDefinition::from_mcp)
            .collect()
    }
    fn call(&mut self, _target: &Target, call: &ToolCall, cancel: &Cancellation) -> Result<Value, AiError> {
        cancel.check()?;
        self.cancel.check()?;
        let result = self.request("tools/call", json!({"name":call.name,"arguments":call.arguments}));
        if self.cancel.is_cancelled() {
            return Err(AiError::new(ErrorKind::Network, "The app operation was interrupted. Inspect completed actions; nothing will be replayed."));
        }
        result
    }
}

pub(super) fn dispatch(app: &mut SheetApp, ctx: &egui::Context, _guard: &mut Guard, pending: Pending) {
    if pending.cancel.is_cancelled() || Instant::now() >= pending.expires {
        control::ControlRequest::abandon(&pending.request.state);
    }
    control::dispatch(app, ctx, pending.request);
}

#[cfg(test)]
mod tests;
