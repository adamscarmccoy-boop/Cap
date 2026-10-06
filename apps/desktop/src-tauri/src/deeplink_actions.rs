use std::sync::Arc;
use tauri::{AppHandle, Manager, Wry};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeepLinkAction {
Record {
screen: Option<String>,
camera: Option<String>,
mic: Option<String>,
system_audio: bool,
},
StopRecording,
OpenSettings,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActionParseFromUrlError {
NotAction,
UnknownAction(String),
InvalidParameters,
}

impl DeepLinkAction {
pub fn try_from_url(url: &Url) -> Result<Self, ActionParseFromUrlError> {
if url.scheme() != "cap-desktop" {
return Err(ActionParseFromUrlError::NotAction);
}

let action_name = url.host_str().unwrap_or("");
match action_name {
"record" => {
let mut screen = None;
let mut camera = None;
let mut mic = None;
let mut system_audio = true;

for (key, val) in url.query_pairs() {
match key.as_ref() {
"screen" => screen = Some(val.into_owned()),
"camera" => camera = Some(val.into_owned()),
"mic" => mic = Some(val.into_owned()),
"system_audio" => system_audio = val != "false",
_ => {}
}
}

Ok(DeepLinkAction::Record {
screen,
camera,
mic,
system_audio,
})
}
"stop" | "stop_recording" => Ok(DeepLinkAction::StopRecording),
"settings" => Ok(DeepLinkAction::OpenSettings),
_ => Err(ActionParseFromUrlError::UnknownAction(action_name.to_string())),
}
}
}

pub struct DeepLinkActionExecutor {
app_handle: Arc<AppHandle<Wry>>,
}

impl DeepLinkActionExecutor {
pub fn new(app_handle: Arc<AppHandle<Wry>>) -> Self {
Self { app_handle }
}

pub fn handle(&self, url: &Url) -> Result<(), ActionParseFromUrlError> {
let action = DeepLinkAction::try_from_url(url)?;
match action {
DeepLinkAction::Record {
screen,
camera,
mic,
system_audio,
} => {
let _ = self.app_handle.emit_all("start-recording-requested", serde_json::json!({
"screen": screen.unwrap_or_else(|| "default".to_string()),
"camera": camera,
"mic": mic,
"systemAudio": system_audio,
}));
}
DeepLinkAction::StopRecording => {
let _ = self.app_handle.emit_all("stop-recording-requested", ());
}
DeepLinkAction::OpenSettings => {
if let Some(window) = self.app_handle.get_window("settings") {
let _ = window.show();
let _ = window.set_focus();
}
}
}
Ok(())
}
}

pub fn handle(app: &AppHandle<Wry>, url: &Url) -> Result<(), ActionParseFromUrlError> {
let executor = DeepLinkActionExecutor::new(Arc::new(app.clone()));
executor.handle(url)
}
