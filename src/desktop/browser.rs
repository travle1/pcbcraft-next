//! Browser startup and asynchronous file IO. The shared Session owns all edits.
use super::*;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(module = "/web/browser.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn open_project(max_bytes: usize) -> std::result::Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    fn download_project(name: &str, bytes: &[u8]) -> std::result::Result<(), JsValue>;
    pub(super) fn set_dirty(dirty: bool);
    fn ready();
    fn failed(message: &str);
}

pub fn launch_web() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(error) = start().await {
            failed(&format!("PCBcraft could not start: {error:?}"));
        }
    });
}

async fn start() -> std::result::Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing browser window"))?;
    let canvas = window
        .document()
        .and_then(|doc| doc.get_element_by_id("pcbcraft"))
        .ok_or_else(|| JsValue::from_str("Missing PCBcraft canvas"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let demo = window
        .location()
        .search()?
        .split(['?', '&'])
        .any(|arg| arg == "demo=1");
    let runner = eframe::WebRunner::new();
    runner
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(move |cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                App::new(
                    cc,
                    Options {
                        demo,
                        ..Default::default()
                    },
                )
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(Into::into)
            }),
        )
        .await?;
    ready();
    // WebRunner retains its callbacks; the application lives for this page's lifetime.
    Ok(())
}

fn error(value: JsValue) -> Error {
    Error::Io(value.as_string().unwrap_or_else(|| format!("{value:?}")))
}

impl App {
    pub(super) fn load(&mut self, _path: Option<PathBuf>) {
        // Open the picker synchronously in the user gesture; await only the result.
        let promise = match open_project(persistence::MAX_BYTES) {
            Ok(promise) => promise,
            Err(e) => {
                self.notice(error(e));
                return;
            }
        };
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let value = wasm_bindgen_futures::JsFuture::from(promise)
                    .await
                    .map_err(error)?;
                if value.is_null() {
                    return Ok(FileResult::Cancelled);
                }
                let values = js_sys::Array::from(&value);
                let name = values.get(0).as_string().ok_or("Missing file name")?;
                let bytes = js_sys::Uint8Array::new(&values.get(1)).to_vec();
                let project = persistence::decode_project(&bytes)?;
                Ok(FileResult::Opened(name.into(), Box::new(project), bytes))
            }
            .await;
            let _ = tx.send(result.unwrap_or_else(FileResult::Error));
        });
    }

    pub(super) fn save(&mut self, _save_as: bool) {
        if self.busy() {
            return;
        }
        let project = self.session.project().clone();
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{}.pc", project.name));
        let result = persistence::encode_project(&project).and_then(|bytes| {
            download_project(&name, &bytes).map_err(error)?;
            Ok(FileResult::Downloaded(name.into()))
        });
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        let _ = tx.send(result.unwrap_or_else(FileResult::Error));
        // A download request does not prove that a durable file exists. Keep dirty state.
    }
}
