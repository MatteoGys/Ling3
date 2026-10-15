//! decoration.rs — Signale à tauri-plugin-decoration que le contenu de la
//! page est monté, pour qu'il active sa zone de déplacement de fenêtre et
//! (sur Windows) ses boutons de contrôle personnalisés. Sans cet événement,
//! le plugin reste inerte : c'est ce qui manquait pour que le glisser
//! fonctionne.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = emit)]
    fn tauri_emit(event: &str, payload: JsValue) -> JsValue;
}

/// À appeler une fois, juste après le montage de l'app (voir main.rs).
pub fn notify_decoration_ready() {
    tauri_emit("decoration-page-load", JsValue::NULL);
}
