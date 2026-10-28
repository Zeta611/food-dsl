use serde::Deserialize;

#[derive(Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Ripples {
    amplitude: f32,
    count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Cup {
    bottom_radius: f32,
    top_radius: f32,
    height: f32,
    /** ripple around the circumference */
    flutes: Ripples,
    /** ripple along the height */
    rings: Ripples,
}

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

/** Samples per ring */
const AROUND: u32 = 256;
/** Samples from bottom to top */
const UP: u32 = 64;

use std::f32::consts::TAU;

/** Radius at height u ∈ [0, 1] and angle t ∈ [0, 1). */
fn radius(cup: &Cup, u: f32, t: f32) -> f32 {
    let base = cup.bottom_radius + (cup.top_radius - cup.bottom_radius) * u;
    let flute = cup.flutes.amplitude * (TAU * cup.flutes.count as f32 * t).sin();
    let ring = cup.rings.amplitude * (TAU * cup.rings.count as f32 * u).sin();
    base + flute + ring
}

pub fn compile(cup: &Cup) -> Geometry {
    let mut positions: Vec<f32> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Push one ring at height u and return the index of the first vertex
    let ring = |positions: &mut Vec<f32>, u: f32| -> u32 {
        let start = positions.len() / 3;
        for i in 0..AROUND {
            let t = i as f32 / AROUND as f32;
            let r = radius(cup, u, t);
            let angle = TAU * t;
            positions.extend([r * angle.cos(), cup.height * u, r * angle.sin()]);
        }
        start.try_into().unwrap()
    };

    // Side crust: two triangles per grid cell between rings j and j + 1
    let mut rows: Vec<u32> = vec![];
    for j in 0..=UP {
        rows.push(ring(&mut positions, j as f32 / UP as f32));
    }
    for j in 0..UP {
        for i in 0..AROUND {
            let next = (i + 1) % AROUND;
            let a = rows.get(j as usize).unwrap() + i;
            let b = rows.get(j as usize + 1).unwrap() + i;
            let c = rows.get(j as usize).unwrap() + next;
            let d = rows.get(j as usize + 1).unwrap() + next;
            // CCW means front
            indices.extend([a, b, c, c, b, d]);
        }
    }

    // Caps: a fan of triangles from the center
    let mut cap = |positions: &mut Vec<f32>, u: bool| {
        let center = (positions.len() / 3) as u32;
        positions.extend([0.0, if u { cup.height } else { 0.0 }, 0.0]);
        let rim = ring(positions, if u { 1.0 } else { 0.0 });
        for i in 0..AROUND {
            let p = rim + i;
            let q = rim + ((i + 1) % AROUND);
            if u {
                indices.extend([center, q, p]);
            } else {
                indices.extend([center, p, q]);
            }
        }
    };
    cap(&mut positions, false);
    cap(&mut positions, true);

    Geometry { positions, indices }
}
