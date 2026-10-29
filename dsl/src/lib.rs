use serde::Deserialize;

pub trait Curve2D {
    /** t ∈ \[0, 1\]  */
    fn path(&self, t: f64) -> (f64, f64);
}

enum BuiltInCurve {
    Circle(Circle),
    Squircle(Squircle),
}

impl Curve2D for BuiltInCurve {
    fn path(&self, t: f64) -> (f64, f64) {
        match self {
            Self::Circle(c) => c.path(t),
            Self::Squircle(c) => c.path(t),
        }
    }
}

struct Circle {
    radius: f64,
}

impl Curve2D for Circle {
    fn path(self: &Circle, t: f64) -> (f64, f64) {
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        (self.radius * c, self.radius * s)
    }
}

impl From<Circle> for BuiltInCurve {
    fn from(c: Circle) -> Self {
        Self::Circle(c)
    }
}

// Maybe use a more parametrizable squircle
// https://www.figma.com/blog/desperately-seeking-squircles/
struct Squircle {
    radius: f64,
}

impl Curve2D for Squircle {
    fn path(&self, t: f64) -> (f64, f64) {
        // https://scipython.com/blog/superellipses/
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        let x = self.radius * c.signum() * c.abs().sqrt();
        let y = self.radius * s.signum() * s.abs().sqrt();
        (x, y)
    }
}

impl From<Squircle> for BuiltInCurve {
    fn from(s: Squircle) -> Self {
        Self::Squircle(s)
    }
}

// Or maybe define a more refined Slice struct, allowing multiple interpolation strategies?
impl<F: Fn(f64) -> (f64, f64)> Curve2D for F {
    fn path(&self, t: f64) -> (f64, f64) {
        self(t)
    }
}

pub trait Texture {
    /** t ∈ \[0, 1\]  */
    fn f(&self, t: f64) -> f64;
}

enum BuiltInTexture {
    None,
    SinusoidalRipples(SinusoidalRipples),
}

impl Texture for BuiltInTexture {
    fn f(&self, t: f64) -> f64 {
        match self {
            Self::None => 0.0,
            Self::SinusoidalRipples(txt) => txt.f(t),
        }
    }
}

struct TexturedCurve<C: DifferentiableCurve2D, T: Texture> {
    base: C,
    texture: T,
}

impl<C: Curve2D, T: Texture> Curve2D for TexturedCurve<C, T> {
    fn path(&self, t: f64) -> (f64, f64) {
        self.base.path(t)
    }
}

struct SinusoidalRipples {
    amplitude: f64,
    count: u64,
}

impl Texture for SinusoidalRipples {
    fn f(&self, t: f64) -> f64 {
        self.amplitude * (std::f64::consts::TAU * self.count as f64 * t).sin()
    }
}

pub trait Sweep {
    fn height(&self) -> f64;

    /** u ∈ \[0, 1\] */
    fn slice(&self, u: f64, interpolation: InterpolationStrategy) -> Box<dyn Curve2D + '_>;
}

pub enum InterpolationStrategy {
    Linear,
}

pub struct BuiltInSweep {
    top: BuiltInCurve,
    bottom: BuiltInCurve,
    height: f64,
}

impl Sweep for BuiltInSweep {
    fn height(&self) -> f64 {
        self.height
    }

    fn slice(&self, u: f64, interpolation: InterpolationStrategy) -> Box<dyn Curve2D + '_> {
        match interpolation {
            InterpolationStrategy::Linear => Box::new(move |t| {
                let (tx, ty) = self.top.path(t);
                let (bx, by) = self.bottom.path(t);
                let x = u * tx + (1.0 - u) * bx;
                let y = u * ty + (1.0 - u) * by;
                (x, y)
            }),
        }
    }
}

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
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
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

impl From<Cup> for BuiltInSweep {
    fn from(cup: Cup) -> Self {
        BuiltInSweep {
            top: Circle {
                radius: cup.top_radius,
            }
            .into(),
            bottom: Circle {
                radius: cup.bottom_radius,
            }
            .into(),
            height: cup.height,
        }
    }
}

pub fn meshify<S: Sweep>(shape: &S) -> Geometry {
    let mut positions: Vec<f32> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Push one slice at height u and return the index of the first vertex
    let slice = |positions: &mut Vec<f32>, u: f32| -> u32 {
        let start = positions.len() / 3;
        let curve = shape.slice(u as f64, InterpolationStrategy::Linear);
        for i in 0..AROUND {
            let t = i as f32 / AROUND as f32;
            //    y
            //    |
            //    |
            //    |-----x
            //   /
            //  /
            // z
            // Should we just make Curve2D lay on the xz-plane?
            let (x, mz) = curve.path(t as f64);
            let y = shape.height() as f32 * u;
            positions.extend([x as f32, y, -mz as f32]);
        }
        start.try_into().unwrap()
    };

    // Side crust: two triangles per grid cell between rings j and j + 1
    let mut rows: Vec<u32> = vec![];
    for j in 0..=UP {
        rows.push(slice(&mut positions, j as f32 / UP as f32));
    }
    for j in 0..UP {
        for i in 0..AROUND {
            let next = (i + 1) % AROUND;
            let a = rows.get(j as usize).unwrap() + i;
            let b = rows.get(j as usize + 1).unwrap() + i;
            let c = rows.get(j as usize).unwrap() + next;
            let d = rows.get(j as usize + 1).unwrap() + next;
            // CCW means front
            indices.extend([a, c, b, c, d, b]);
        }
    }

    // Caps: a fan of triangles from the center
    let mut cap = |positions: &mut Vec<f32>, u: bool| {
        let center = (positions.len() / 3) as u32;
        positions.extend([0.0, if u { shape.height() as f32 } else { 0.0 }, 0.0]);
        let rim = slice(positions, if u { 1.0 } else { 0.0 });
        for i in 0..AROUND {
            let p = rim + i;
            let q = rim + ((i + 1) % AROUND);
            if u {
                indices.extend([center, p, q]);
            } else {
                indices.extend([center, q, p]);
            }
        }
    };
    cap(&mut positions, false);
    cap(&mut positions, true);

    Geometry { positions, indices }
}

/*
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
*/
