mod utils;

use fldsmdfr::{Cup, Geometry, Path3D};
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
pub struct ToolpathHandle {
    positions: Vec<f32>,
}

#[wasm_bindgen]
impl ToolpathHandle {
    pub fn positions_len(&self) -> usize {
        self.positions.len()
    }

    pub fn positions_ptr(&self) -> *const f32 {
        self.positions.as_ptr()
    }
}

impl From<Path3D> for ToolpathHandle {
    fn from(p: Path3D) -> Self {
        ToolpathHandle {
            positions: p.into_flattened(),
        }
    }
}

#[wasm_bindgen]
pub fn compile(cup: JsValue) -> Result<GeometryHandle, JsError> {
    utils::set_panic_hook();
    Ok(serde_wasm_bindgen::from_value::<Cup>(cup)?.meshify().into())
}

#[wasm_bindgen]
pub fn pathify(cup: JsValue) -> Result<ToolpathHandle, JsError> {
    utils::set_panic_hook();
    Ok(serde_wasm_bindgen::from_value::<Cup>(cup)?.pathify().into())
}
