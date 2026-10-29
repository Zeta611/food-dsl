mod utils;

use fldsmdfr::{Cup, Geometry, TexturedSweep};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GeometryHandle {
    positions: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl GeometryHandle {
    pub fn positions_len(&self) -> usize {
        self.positions.len()
    }

    pub fn indices_len(&self) -> usize {
        self.indices.len()
    }

    pub fn positions_ptr(&self) -> *const f32 {
        self.positions.as_ptr()
    }

    pub fn indices_ptr(&self) -> *const u32 {
        self.indices.as_ptr()
    }
}

impl From<Geometry> for GeometryHandle {
    fn from(Geometry { positions, indices }: Geometry) -> Self {
        GeometryHandle { positions, indices }
    }
}

#[wasm_bindgen]
pub fn compile(cup: JsValue) -> Result<GeometryHandle, JsError> {
    utils::set_panic_hook();
    let sweep = TexturedSweep::from(serde_wasm_bindgen::from_value::<Cup>(cup)?);
    Ok(fldsmdfr::meshify(&sweep).into())
}
