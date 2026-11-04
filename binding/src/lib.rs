mod utils;

use fldsmdfr::{Cup, Geometry};
use js_sys::{Float32Array, Uint32Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct Mesh {
    pub positions: Float32Array,
    pub indices: Uint32Array,
}

impl From<Geometry> for Mesh {
    fn from(Geometry { positions, indices }: Geometry) -> Self {
        Mesh {
            positions: Float32Array::from(&positions[..]),
            indices: Uint32Array::from(&indices[..]),
        }
    }
}

#[wasm_bindgen]
pub fn shell_mesh(cup: JsValue) -> Result<Mesh, JsError> {
    utils::set_panic_hook();
    Ok(serde_wasm_bindgen::from_value::<Cup>(cup)?
        .shell_mesh()
        .into())
}

#[wasm_bindgen]
pub fn shell_path(cup: JsValue) -> Result<Float32Array, JsError> {
    utils::set_panic_hook();
    Ok(Float32Array::from(
        serde_wasm_bindgen::from_value::<Cup>(cup)?
            .shell_path()
            .as_flattened(),
    ))
}

#[wasm_bindgen]
pub fn infill_mesh(cup: JsValue, lattice: JsValue) -> Result<Mesh, JsError> {
    utils::set_panic_hook();
    Ok(serde_wasm_bindgen::from_value::<Cup>(cup)?
        .infill_mesh(&serde_wasm_bindgen::from_value(lattice)?)
        .into())
}
