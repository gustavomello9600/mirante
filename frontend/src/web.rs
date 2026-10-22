use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedItem {
    pub query: String,
    pub title: String,
    pub saved_at: String,
}

pub fn document_element() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

pub fn get_theme() -> String {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("mirante-theme").ok().flatten())
        .unwrap_or_else(|| "light".to_string())
}

pub fn set_theme(theme: &str) {
    if let Some(element) = document_element() {
        let _ = element.set_attribute("data-theme", theme);
    }
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item("mirante-theme", theme);
    }
}

pub fn read_query_param() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    let trimmed = search.trim_start_matches('?');
    for pair in trimmed.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some("q") {
            let raw = parts.next().unwrap_or("");
            return js_sys::decode_uri_component(raw)
                .ok()
                .and_then(|value| value.as_string())
                .filter(|value| !value.trim().is_empty());
        }
    }
    None
}

pub fn push_query(query: &str) {
    let encoded = js_sys::encode_uri_component(query)
        .as_string()
        .unwrap_or_else(|| query.to_string());
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let url = format!("?q={encoded}");
            let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&url));
        }
        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

pub async fn copy(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "sem window".to_string())?;
    let clipboard = window.navigator().clipboard();
    let promise = clipboard.write_text(text);
    if JsFuture::from(promise).await.is_ok() {
        return Ok(());
    }
    fallback_copy(text)
}

fn fallback_copy(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "sem window".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "sem documento".to_string())?;
    let element = document
        .create_element("textarea")
        .map_err(|_| "sem textarea".to_string())?;
    let textarea = element
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .map_err(|_| "textarea inválido".to_string())?;
    textarea.set_value(text);
    textarea.set_attribute("readonly", "").ok();
    textarea.set_attribute("aria-hidden", "true").ok();
    textarea
        .set_attribute(
            "style",
            "position:fixed;top:-1000px;left:-1000px;opacity:0;",
        )
        .ok();
    let body = document.body().ok_or_else(|| "sem body".to_string())?;
    body.append_child(&textarea)
        .map_err(|_| "sem append".to_string())?;
    textarea.select();
    let copied = document
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()
        .and_then(|doc| doc.exec_command("copy").ok())
        .unwrap_or(false);
    textarea.remove();
    if copied {
        Ok(())
    } else {
        Err("Não foi possível copiar".to_string())
    }
}

pub fn saved_items() -> Vec<SavedItem> {
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return Vec::new();
    };
    let Some(raw) = storage.get_item("mirante-saved").ok().flatten() else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_items(items: &[SavedItem]) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        if let Ok(raw) = serde_json::to_string(items) {
            let _ = storage.set_item("mirante-saved", &raw);
        }
    }
}

pub fn toggle_saved(query: &str, title: &str) -> bool {
    let mut items = saved_items();
    if let Some(index) = items.iter().position(|item| item.query == query) {
        items.remove(index);
        save_items(&items);
        false
    } else {
        items.insert(
            0,
            SavedItem {
                query: query.to_string(),
                title: title.to_string(),
                saved_at: js_sys::Date::new_0().to_iso_string().into(),
            },
        );
        save_items(&items);
        true
    }
}

pub fn remove_saved(query: &str) {
    let mut items = saved_items();
    items.retain(|item| item.query != query);
    save_items(&items);
}

pub fn format_time(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    date.to_locale_time_string("pt-BR")
        .as_string()
        .unwrap_or_else(|| value.to_string())
}

pub fn format_date(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    date.to_locale_date_string("pt-BR", &JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_else(|| value.to_string())
}

pub fn relative_time(value: &str) -> String {
    if value.is_empty() {
        return "sem data".to_string();
    }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    let now = js_sys::Date::now();
    let diff = (now - date.get_time()).max(0.0);
    let minutes = (diff / 60000.0).round() as i64;
    match minutes {
        0..=2 => "agora".to_string(),
        3..=59 => format!("há {minutes} min"),
        60..=1439 => format!("há {} h", minutes / 60),
        _ => format_date(value),
    }
}
