use serde::Deserialize;

// TODO: See if using glam makes life easier
// https://github.com/bitshifter/glam-rs
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

// TODO: Change name
#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum Profile {
    Circle,
    Squircle,
}

// TODO: Change name
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
}

type CupSweep = TexturedSweep<Base, Base, SinusoidalRipples, SinusoidalRipples, SinusoidalRipples>;

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Grid {
    spacing: f64,
}

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct Honeycomb {
    spacing: f64,
}

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum LatticePreset {
    Grid(Grid),
    Honeycomb(Honeycomb),
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub min: (f64, f64),
    pub max: (f64, f64),
}

impl Rect {
    /** Smallest rectangle containing every point */
    fn bounding(points: &[(f64, f64)]) -> Self {
        let empty = Rect {
            min: (f64::INFINITY, f64::INFINITY),
            max: (f64::NEG_INFINITY, f64::NEG_INFINITY),
        };
        points
            .iter()
            .fold(empty, |Rect { min, max }, &(x, y)| Rect {
                min: (min.0.min(x), min.1.min(y)),
                max: (max.0.max(x), max.1.max(y)),
            })
    }
}

pub trait Lattice {
    /** Centers of cells overlapping with `window` */
    fn centers(&self, window: Rect) -> Vec<(f64, f64)>;

    /** Corners of the "seed" cell at the origin (CCW) */
    fn seed(&self) -> Vec<(f64, f64)>;
}

/** k's such that lo ≤ k * step ≤ hi, padded by one on each side */
fn steps(lo: f64, hi: f64, step: f64) -> std::ops::RangeInclusive<i64> {
    (((lo / step).floor() as i64) - 1)..=(((hi / step).ceil() as i64) + 1)
}

impl Lattice for Grid {
    fn centers(&self, window: Rect) -> Vec<(f64, f64)> {
        let s = self.spacing;
        let xs = steps(window.min.0, window.max.0, s);
        let ys = steps(window.min.1, window.max.1, s);

        let mut out = vec![];
        for j in ys {
            for i in xs.clone() {
                out.push((i as f64 * s, j as f64 * s));
            }
        }
        out
    }

    fn seed(&self) -> Vec<(f64, f64)> {
        let h = self.spacing / 2.0;
        vec![(h, -h), (h, h), (-h, h), (-h, -h)]
    }
}

impl Lattice for Honeycomb {
    fn centers(&self, window: Rect) -> Vec<(f64, f64)> {
        let s = self.spacing;
        // h = (3/2)r = (√3/2)s where r is the length of the edge (s/√3)
        let h = s * 3f64.sqrt() / 2.0;
        let xs = steps(window.min.0, window.max.0, s);
        let ys = steps(window.min.1, window.max.1, h);

        let mut out = vec![];
        for j in ys {
            let shift = if j % 2 != 0 { s / 2.0 } else { 0.0 };
            for i in xs.clone() {
                out.push((i as f64 * s + shift, j as f64 * h));
            }
        }
        out
    }

    fn seed(&self) -> Vec<(f64, f64)> {
        use std::f64::consts::{FRAC_PI_3, FRAC_PI_6};
        // s/√3 (length of the edge)
        let r = self.spacing / 3f64.sqrt();
        (0..6)
            .map(|k| {
                let (s, c) = (FRAC_PI_6 + k as f64 * FRAC_PI_3).sin_cos();
                (r * c, r * s)
            })
            .collect()
    }
}

impl Lattice for LatticePreset {
    fn centers(&self, window: Rect) -> Vec<(f64, f64)> {
        match self {
            LatticePreset::Grid(g) => g.centers(window),
            LatticePreset::Honeycomb(h) => h.centers(window),
        }
    }

    fn seed(&self) -> Vec<(f64, f64)> {
        match self {
            LatticePreset::Grid(g) => g.seed(),
            LatticePreset::Honeycomb(h) => h.seed(),
        }
    }
}

pub type Segment = ((f64, f64), (f64, f64));

/** Cell walls covering `window` */
fn walls<L: Lattice>(lattice: &L, window: Rect) -> Vec<Segment> {
    let seed = lattice.seed();
    let n = seed.len();

    // Only count a shared wall once
    let half: Vec<Segment> = (0..n)
        .map(|k| (seed[k], seed[(k + 1) % n]))
        .filter(|&((ax, ay), (bx, by))| {
            let (dx, dy) = (bx - ax, by - ay);
            let eps = 1e-9 * dx.hypot(dy);
            // rightward || straight-up
            dx > eps || (dx.abs() <= eps && dy > 0.0)
        })
        .collect();

    let mut out = vec![];
    for (cx, cy) in lattice.centers(window) {
        for ((ax, ay), (bx, by)) in &half {
            out.push(((cx + ax, cy + ay), (cx + bx, cy + by)))
        }
    }
    out
}

fn cross((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> f64 {
    ax * by - ay * bx
}

/** +x crosses the boundary an odd number of times
 * https://en.wikipedia.org/wiki/Point_in_polygon#Ray_casting_algorithm */
fn inside(polygon: &[(f64, f64)], (px, py): (f64, f64)) -> bool {
    let n = polygon.len();
    let crossings = (0..n)
        .filter(|&k| {
            let (ax, ay) = polygon[k];
            let (bx, by) = polygon[(k + 1) % n];
            (ay > py) != (by > py) && px < ax + (py - ay) / (by - ay) * (bx - ax)
        })
        .count();
    crossings % 2 == 1
}

/** Pieces of edge `p`-`q` inside `polygon` */
fn clip(polygon: &[(f64, f64)], (p, q): Segment) -> Vec<Segment> {
    let r = (q.0 - p.0, q.1 - p.1);
    // at(0) = p, at(1) = q
    let at = |t: f64| (p.0 + t * r.0, p.1 + t * r.1);

    // Parameters t where p + tr meets the edges, plus both ends
    let mut ts = vec![0.0, 1.0];
    let n = polygon.len();
    for k in 0..n {
        let a = polygon[k];
        let b = polygon[(k + 1) % n];
        let e = (b.0 - a.0, b.1 - a.1);
        let denom = cross(r, e);
        if denom == 0.0 {
            // parallel (cross product is zero)
            continue;
        }
        // Cramer's rule
        //   p + tr = a + ue
        // ⇔ [r -e][t u]ᵗ = [a-p]
        let ap = (a.0 - p.0, a.1 - p.1);
        let (t, u) = (cross(ap, e) / denom, cross(ap, r) / denom);
        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            ts.push(t);
        }
    }

    ts.sort_by(f64::total_cmp); // [0.0, ..., 1.0]
    // Between consecutive crossings the segment is wholly in or out
    ts.windows(2)
        /* filter out the zero-length cut when the intersection occurs at the vertex */
        .filter(|w| w[1] > w[0])
        .filter(|w| inside(polygon, at((w[0] + w[1]) / 2.0)))
        .map(|w| (at(w[0]), at(w[1])))
        .collect()
}

/** Lattice walls inside a closed polygon */
fn fill_polygon<L: Lattice>(lattice: &L, polygon: &[(f64, f64)]) -> Vec<Segment> {
    walls(lattice, Rect::bounding(polygon))
        .into_iter()
        .flat_map(|wall| clip(polygon, wall))
        .collect()
}

/** Lattice walls inside a closed curve, clipped against its sampled polygon */
pub fn fill<L: Lattice>(lattice: &L, region: &dyn Curve2D) -> Vec<Segment> {
    let sampled_region: Vec<(f64, f64)> = (0..AROUND)
        .map(|i| region.path(i as f64 / AROUND as f64))
        .collect();
    fill_polygon(lattice, &sampled_region)
}

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

pub type Path3D = Vec<[f32; 3]>;

/** Samples per slice */
const AROUND: u32 = 256;
/** Samples from bottom to top */
const UP: u32 = 64;

impl Cup {
    pub fn shell_mesh(self) -> Geometry {
        let sweep = CupSweep::from(self);
        let mut mesh = MeshBuilder::default();
        mesh.side(&sweep, 0.0, 1.0, true);
        mesh.cap(&sweep, 0.0, false);
        mesh.into()
    }

    pub fn shell_path(self) -> Path3D {
        let sweep = CupSweep::from(self);
        let mut path = PathBuilder::default();
        path.side(&sweep, 0.0, 1.0);
        path.into()
    }

    pub fn infill_mesh(self, lattice: &LatticePreset) -> Geometry {
        let sweep = CupSweep::from(self);
        let mut mesh = MeshBuilder::default();
        let h = sweep.height();
        for j in 0..UP {
            let (u0, u1) = (j as f64 / UP as f64, (j + 1) as f64 / UP as f64);
            let (y0, y1) = ((h * u0) as f32, (h * u1) as f32);
            for (a, b) in fill(lattice, &*sweep.slice(u0)) {
                mesh.wall(a, b, y0, y1);
            }
        }
        mesh.into()
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

    fn wall(&mut self, (ax, ay): (f64, f64), (bx, by): (f64, f64), y0: f32, y1: f32) {
        let start = (self.positions.len() / 3) as u32;
        // slice y becomes -z
        let (ax, az, bx, bz) = (ax as f32, -ay as f32, bx as f32, -by as f32);
        self.positions
            .extend([ax, y0, az, bx, y0, bz, bx, y1, bz, ax, y1, az]);
        self.indices
            .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    }

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

#[derive(Default)]
struct PathBuilder {
    positions: Vec<[f32; 3]>,
}

impl PathBuilder {
    /** Push one slice at height u */
    fn slice<S: Sweep>(&mut self, shape: &S, u: f64) {
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
            let (x, mz) = curve.path(t);
            self.positions.push([x as f32, y, -mz as f32]);
        }
    }

    /** Side wall from u0 to u1 */
    fn side<S: Sweep>(&mut self, shape: &S, u0: f64, u1: f64) {
        (0..=UP).for_each(|j| self.slice(shape, u0 + (u1 - u0) * j as f64 / UP as f64));
    }
}

impl From<PathBuilder> for Path3D {
    fn from(PathBuilder { positions }: PathBuilder) -> Self {
        positions
    }
}
