use serde::Deserialize;

pub trait Curve2D {
    /** t ∈ \[0, 1\]  */
    fn path(&self, t: f64) -> (f64, f64);
}

pub trait DifferentiableCurve2D: Curve2D {
    /** ⟨ẏ(t), −ẋ(t)⟩ / ‖ċ(t)‖ at t ∈ \[0, 1\] */
    fn normal(&self, t: f64) -> (f64, f64);
}

fn normalize((x, y): (f64, f64)) -> (f64, f64) {
    let len = x.hypot(y);
    (x / len, y / len)
}

pub struct Circle {
    radius: f64,
}

impl Curve2D for Circle {
    fn path(self: &Circle, t: f64) -> (f64, f64) {
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        (self.radius * c, self.radius * s)
    }
}

impl DifferentiableCurve2D for Circle {
    fn normal(&self, t: f64) -> (f64, f64) {
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        (c, s)
    }
}

// Maybe use a more parametrizable squircle
// https://www.figma.com/blog/desperately-seeking-squircles/
pub struct Squircle {
    radius: f64,
}

impl Curve2D for Squircle {
    // https://en.wikipedia.org/wiki/Squircle in polar form
    fn path(&self, t: f64) -> (f64, f64) {
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        let r = self.radius / (c.powi(4) + s.powi(4)).powf(0.25);
        (r * c, r * s)
    }
}

impl DifferentiableCurve2D for Squircle {
    fn normal(&self, t: f64) -> (f64, f64) {
        let (s, c) = (std::f64::consts::TAU * t).sin_cos();
        normalize((c.powi(3), s.powi(3)))
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

struct TexturedCurve<C: DifferentiableCurve2D, T: Texture> {
    base: C,
    texture: T,
}

impl<C: DifferentiableCurve2D, T: Texture> Curve2D for TexturedCurve<C, T> {
    fn path(&self, t: f64) -> (f64, f64) {
        let (x, y) = self.base.path(t);
        let (nx, ny) = self.base.normal(t);
        let f = self.texture.f(t);
        (x + f * nx, y + f * ny)
    }
}

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct SinusoidalRipples {
    amplitude: f64,
    count: u64,
}

impl Texture for SinusoidalRipples {
    fn f(&self, t: f64) -> f64 {
        self.amplitude * (std::f64::consts::TAU * self.count as f64 * t).sin()
    }
}

pub struct NullTexture {}

impl Texture for NullTexture {
    fn f(&self, _: f64) -> f64 {
        0.0
    }
}

pub trait Sweep {
    fn height(&self) -> f64;

    /** u ∈ \[0, 1\] */
    fn slice(&self, u: f64) -> Box<dyn Curve2D + '_>;
}

pub enum InterpolationStrategy<T: Texture> {
    Lerp(T),
}

pub struct TexturedSweep<
    CT: DifferentiableCurve2D,
    CB: DifferentiableCurve2D,
    TT: Texture,
    TB: Texture,
    TI: Texture,
> {
    top: TexturedCurve<CT, TT>,
    bottom: TexturedCurve<CB, TB>,
    height: f64,
    interpolation: InterpolationStrategy<TI>,
}

impl<CT: DifferentiableCurve2D, CB: DifferentiableCurve2D, TT: Texture, TB: Texture, TI: Texture>
    Sweep for TexturedSweep<CT, CB, TT, TB, TI>
{
    fn height(&self) -> f64 {
        self.height
    }

    fn slice(&self, u: f64) -> Box<dyn Curve2D + '_> {
        match &self.interpolation {
            InterpolationStrategy::Lerp(txt) => {
                let offset = txt.f(u);
                Box::new(move |t| {
                    let (tx, ty) = self.top.path(t);
                    let (bx, by) = self.bottom.path(t);
                    let (x, y) = (u * tx + (1.0 - u) * bx, u * ty + (1.0 - u) * by);

                    let (tnx, tny) = self.top.base.normal(t);
                    let (bnx, bny) = self.bottom.base.normal(t);
                    let (nx, ny) =
                        normalize((u * tnx + (1.0 - u) * bnx, u * tny + (1.0 - u) * bny));

                    (x + offset * nx, y + offset * ny)
                })
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Cup {
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    /** ripple around the circumference */
    flutes: SinusoidalRipples,
    /** ripple along the height */
    rings: SinusoidalRipples,
}

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

/** Samples per ring */
const AROUND: u32 = 256;
/** Samples from bottom to top */
const UP: u32 = 64;

impl From<Cup>
    for TexturedSweep<Squircle, Circle, SinusoidalRipples, SinusoidalRipples, SinusoidalRipples>
{
    fn from(cup: Cup) -> Self {
        TexturedSweep {
            top: TexturedCurve {
                base: Squircle {
                    radius: cup.top_radius,
                },
                texture: cup.flutes,
            },
            bottom: TexturedCurve {
                base: Circle {
                    radius: cup.bottom_radius,
                },
                texture: cup.flutes,
            },
            height: cup.height,
            interpolation: InterpolationStrategy::Lerp(cup.rings),
        }
    }
}

pub fn meshify<S: Sweep>(shape: &S) -> Geometry {
    let mut positions: Vec<f32> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Push one slice at height u and return the index of the first vertex
    let slice = |positions: &mut Vec<f32>, u: f32| -> u32 {
        let start = positions.len() / 3;
        let curve = shape.slice(u as f64);
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
