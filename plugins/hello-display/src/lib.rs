use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

/// Called by the Shell after it receives an IpcResponse.
/// `data` is the response payload as a JS object.
#[wasm_bindgen]
pub fn render(data: JsValue) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;

    let canvas = document
        .get_element_by_id("plugin-canvas")
        .ok_or_else(|| JsValue::from_str("#plugin-canvas not found"))?;

    // Extract greeting string from payload
    let greeting = Reflect::get(&data, &JsValue::from_str("greeting"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "Hello from the Display Plugin!".to_string());

    // Build greeting card
    let card: HtmlElement = document
        .create_element("div")?
        .unchecked_into();
    set_styles(&card, &[
        ("border", "2px solid #C84B31"),
        ("background", "#F6F4F0"),
        ("padding", "16px 24px"),
        ("border-radius", "8px"),
        ("margin", "8px 0"),
        ("font-family", "system-ui, sans-serif"),
    ])?;

    // Greeting text
    let greeting_el: HtmlElement = document.create_element("p")?.unchecked_into();
    greeting_el.set_inner_text(&greeting);
    set_styles(&greeting_el, &[
        ("font-size", "18px"),
        ("color", "#171717"),
        ("margin", "0 0 8px"),
        ("font-weight", "500"),
    ])?;
    card.append_child(&greeting_el)?;

    // Plugin label
    let label_el: HtmlElement = document.create_element("p")?.unchecked_into();
    label_el.set_inner_text("hello-display · wasm-bindgen · wasm32-unknown-unknown");
    set_styles(&label_el, &[
        ("font-size", "11px"),
        ("color", "rgba(23,23,23,0.45)"),
        ("margin", "0"),
        ("letter-spacing", "0.03em"),
    ])?;
    card.append_child(&label_el)?;

    canvas.append_child(&card)?;
    Ok(())
}

fn set_styles(el: &HtmlElement, styles: &[(&str, &str)]) -> Result<(), JsValue> {
    let style = el.style();
    for (prop, val) in styles {
        style.set_property(prop, val)?;
    }
    Ok(())
}
