//! Browser file picker: a hidden `<input type="file">` driven from the
//! Open screen's button. wasm has no drag-and-drop on the winit canvas,
//! so this is the file ingress path for the web build.
//!
//! `trigger` clicks the input; the `change` handler reads the picked file
//! through `FileReader` and hands its bytes back to the event loop as
//! `UserEvent::FileLoaded`. The input and both closures live in a
//! thread_local for the whole session (single-threaded wasm).

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use winit::event_loop::EventLoopProxy;

use crate::app::UserEvent;

thread_local! {
    static PICKER: RefCell<Option<web_sys::HtmlInputElement>> = const { RefCell::new(None) };
    // The closures must outlive the event they wait on; dropping them
    // would silently detach the handlers.
    static CLOSURES: RefCell<Vec<Closure<dyn FnMut()>>> = const { RefCell::new(Vec::new()) };
}

/// Lazily create the hidden input and wire its `change` handler.
fn ensure_input(proxy: EventLoopProxy<UserEvent>) -> Option<web_sys::HtmlInputElement> {
    PICKER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return slot.clone();
        }
        let document = web_sys::window()?.document()?;
        let input = document
            .create_element("input")
            .ok()?
            .dyn_into::<web_sys::HtmlInputElement>()
            .ok()?;
        input.set_type("file");
        input.set_accept(".nest,application/octet-stream");
        input.style().set_property("display", "none").ok()?;
        document.body()?.append_child(&input).ok()?;

        let on_change = Closure::wrap(Box::new(move || {
            PICKER.with(|slot| {
                let Some(input) = slot.borrow().as_ref().cloned() else {
                    return;
                };
                let Some(file) = input.files().and_then(|list| list.get(0)) else {
                    return;
                };
                let name = file.name();
                let reader = match web_sys::FileReader::new() {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("FileReader init failed: {e:?}");
                        return;
                    }
                };
                let proxy = proxy.clone();
                // FileReader is a JsValue handle: clone it for the closure so
                // the outer binding stays usable for set_onload/read.
                let reader_for_load = reader.clone();
                let on_load = Closure::wrap(Box::new(move || {
                    if let Ok(buf) = reader_for_load.result() {
                        let array = js_sys::Uint8Array::new(&buf);
                        let mut bytes = vec![0u8; array.length() as usize];
                        array.copy_to(&mut bytes);
                        let _ = proxy.send_event(UserEvent::FileLoaded {
                            name: name.clone(),
                            bytes,
                        });
                    }
                }) as Box<dyn FnMut()>);
                reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                CLOSURES.with(|c| c.borrow_mut().push(on_load));
                if let Err(e) = reader.read_as_array_buffer(&file) {
                    log::error!("FileReader read failed: {e:?}");
                }
            });
        }) as Box<dyn FnMut()>);
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        CLOSURES.with(|c| c.borrow_mut().push(on_change));

        *slot = Some(input.clone());
        slot.clone()
    })
}

/// Open the picker (called from `about_to_wait` when the Open screen's
/// button set the pick request flag).
pub fn trigger(proxy: EventLoopProxy<UserEvent>) {
    match ensure_input(proxy) {
        Some(input) => input.click(),
        None => log::error!("file picker unavailable: no document"),
    }
}
