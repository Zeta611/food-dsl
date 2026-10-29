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

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum Profile {
    Circle,
    Squircle,
}

pub enum Base {
    Circle(Circle),
    Squircle(Squircle),
}

impl Base {
    fn new(profile: Profile, radius: f64) -> Self {
        match profile {
            Profile::Circle => Base::Circle(Circle { radius }),
            Profile::Squircle => Base::Squircle(Squircle { radius }),
        }
    }
}

impl Curve2D for Base {
    fn path(&self, t: f64) -> (f64, f64) {
        match self {
            Base::Circle(c) => c.path(t),
            Base::Squircle(s) => s.path(t),
        }
    }
}

impl DifferentiableCurve2D for Base {
    fn normal(&self, t: f64) -> (f64, f64) {
        match self {
            Base::Circle(c) => c.normal(t),
            Base::Squircle(s) => s.normal(t),
        }
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

pub struct ConstantTexture(f64);

impl Texture for ConstantTexture {
    fn f(&self, _: f64) -> f64 {
        self.0
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

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Cup {
    bottom_radius: f64,
    top_radius: f64,
    bottom: Profile,
    top: Profile,
    height: f64,
    /** ripple around the circumference */
    flutes: SinusoidalRipples,
    /** ripple along the height */
    rings: SinusoidalRipples,
    /** None means solid */
    cavity: Option<Cavity>,
}

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Cavity {
    bottom: Profile,
    top: Profile,
    wall: f64,
    floor: f64,
}

pub struct Carved<O: Sweep, I: Sweep> {
    pub outer: O,
    pub inner: I,
    pub floor: f64,
}

pub type CupSweep =
    TexturedSweep<Base, Base, SinusoidalRipples, SinusoidalRipples, SinusoidalRipples>;
pub type CavitySweep = TexturedSweep<Base, Base, ConstantTexture, ConstantTexture, NullTexture>;

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

/** Samples per slice */
const AROUND: u32 = 256;
/** Samples from bottom to top */
const UP: u32 = 64;

impl Cup {
    pub fn meshify(self) -> Geometry {
        match self.cavity {
            None => meshify(&CupSweep::from(self)),
            Some(cavity) => meshify_carved(&self.carve(cavity)),
        }
    }

    fn carve(self, cavity: Cavity) -> Carved<CupSweep, CavitySweep> {
        let inset = cavity.wall + self.flutes.amplitude.abs() + self.rings.amplitude.abs();
        Carved {
            outer: CupSweep::from(self),
            inner: TexturedSweep {
                top: TexturedCurve {
                    base: Base::new(cavity.top, self.top_radius),
                    texture: ConstantTexture(-inset),
                },
                bottom: TexturedCurve {
                    base: Base::new(cavity.bottom, self.bottom_radius),
                    texture: ConstantTexture(-inset),
                },
                height: self.height,
                interpolation: InterpolationStrategy::Lerp(NullTexture {}),
            },
            floor: cavity.floor.clamp(0.0, self.height),
        }
    }
}

impl From<Cup> for CupSweep {
    fn from(cup: Cup) -> Self {
        TexturedSweep {
            top: TexturedCurve {
                base: Base::new(cup.top, cup.top_radius),
                texture: cup.flutes,
            },
            bottom: TexturedCurve {
                base: Base::new(cup.bottom, cup.bottom_radius),
                texture: cup.flutes,
            },
            height: cup.height,
            interpolation: InterpolationStrategy::Lerp(cup.rings),
        }
    }
}

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<f32>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    /** Push one slice at height u and return the index of the first vertex */
    fn slice<S: Sweep>(&mut self, shape: &S, u: f64) -> u32 {
        let start = (self.positions.len() / 3) as u32;
        let curve = shape.slice(u);
        let y = (shape.height() * u) as f32;
        for i in 0..AROUND {
            let t = i as f64 / AROUND as f64;
            //    y
            //    |
            //    |
            //    |-----x
            //   /
            //  /
            // z
            // Should we just make Curve2D lay on the xz-plane?
            let (x, mz) = curve.path(t);
            self.positions.extend([x as f32, y, -mz as f32]);
        }
        start
    }

    /** Two triangles per cell between slices a and b */
    fn stitch(&mut self, a: u32, b: u32) {
        for i in 0..AROUND {
            let next = (i + 1) % AROUND;
            let (a0, a1, b0, b1) = (a + i, a + next, b + i, b + next);
            // CCW means front
            self.indices.extend([a0, a1, b0, a1, b1, b0]);
        }
    }

    /** Side wall from u0 to u1; outward = false faces it toward the axis */
    fn side<S: Sweep>(&mut self, shape: &S, u0: f64, u1: f64, outward: bool) {
        let rows: Vec<u32> = (0..=UP)
            .map(|j| self.slice(shape, u0 + (u1 - u0) * j as f64 / UP as f64))
            .collect();
        for w in rows.windows(2) {
            if outward {
                self.stitch(w[0], w[1]);
            } else {
                self.stitch(w[1], w[0]);
            }
        }
    }

    /** A fan of triangles from the center of the slice at height u */
    fn cap<S: Sweep>(&mut self, shape: &S, u: f64, up: bool) {
        let center = (self.positions.len() / 3) as u32;
        self.positions
            .extend([0.0, (shape.height() * u) as f32, 0.0]);
        let rim = self.slice(shape, u);
        for i in 0..AROUND {
            let p = rim + i;
            let q = rim + (i + 1) % AROUND;
            if up {
                self.indices.extend([center, p, q]);
            } else {
                self.indices.extend([center, q, p]);
            }
        }
    }
}

impl From<MeshBuilder> for Geometry {
    fn from(MeshBuilder { positions, indices }: MeshBuilder) -> Self {
        Geometry { positions, indices }
    }
}

pub fn meshify<S: Sweep>(shape: &S) -> Geometry {
    let mut mesh = MeshBuilder::default();
    mesh.side(shape, 0.0, 1.0, true);
    mesh.cap(shape, 0.0, false);
    mesh.cap(shape, 1.0, true);
    mesh.into()
}

pub fn meshify_carved<O: Sweep, I: Sweep>(
    Carved {
        outer,
        inner,
        floor,
    }: &Carved<O, I>,
) -> Geometry {
    let u_floor = floor / inner.height();

    let mut mesh = MeshBuilder::default();
    mesh.side(outer, 0.0, 1.0, true);
    mesh.cap(outer, 0.0, false);
    mesh.side(inner, u_floor, 1.0, false);
    mesh.cap(inner, u_floor, true);
    let (o, i) = (mesh.slice(outer, 1.0), mesh.slice(inner, 1.0));
    mesh.stitch(o, i);
    mesh.into()
}
