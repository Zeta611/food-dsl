use geo::algorithm::line_measures::{Distance, Euclidean, InterpolatableLine};
use geo::{
    BooleanOps, BoundingRect, Contains, Coord, Line, LineString, LinesIter, MultiLineString,
    MultiPoint, Point, Polygon, Rect, Translate, Vector2DOps, Winding, coord, line_string, point,
};
use serde::Deserialize;
use std::f64::consts::TAU;
use std::fmt::Debug;
use std::ops::RangeInclusive;

pub trait Curve2D {
    /// t ∈ \[0, 1\]
    fn path(&self, t: f64) -> Coord;
}

pub trait DifferentiableCurve2D: Curve2D {
    /// ⟨ẏ(t), −ẋ(t)⟩ / ‖ċ(t)‖ at t ∈ \[0, 1\]
    fn normal(&self, t: f64) -> Coord;
}

#[track_caller]
fn check_range<T: PartialOrd + Debug>(x: T, range: RangeInclusive<T>) {
    debug_assert!(range.contains(&x), "expected {x:?} to be in {range:?}");
}

pub struct Circle {
    radius: f64,
}

impl Curve2D for Circle {
    fn path(self: &Circle, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        let (s, c) = (TAU * t).sin_cos();
        let r = self.radius;
        coord!(x: r * c, y: r * s)
    }
}

impl DifferentiableCurve2D for Circle {
    fn normal(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        let (s, c) = (TAU * t).sin_cos();
        coord!(x: c, y: s)
    }
}

// Maybe use a more parametrizable squircle
// https://www.figma.com/blog/desperately-seeking-squircles/
pub struct Squircle {
    radius: f64,
}

impl Curve2D for Squircle {
    // https://en.wikipedia.org/wiki/Squircle in polar form
    fn path(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        let (s, c) = (TAU * t).sin_cos();
        let r = self.radius / (c.powi(4) + s.powi(4)).powf(0.25);
        coord!(x: r * c, y: r * s)
    }
}

impl DifferentiableCurve2D for Squircle {
    fn normal(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        let (s, c) = (TAU * t).sin_cos();
        let Some(n) = coord!(x: c.powi(3), y: s.powi(3)).try_normalize() else {
            panic!("Differentiable curve failed to calculate a normal vector for ({s}, {c})")
        };
        n
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
    fn path(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        match self {
            Base::Circle(c) => c.path(t),
            Base::Squircle(s) => s.path(t),
        }
    }
}

impl DifferentiableCurve2D for Base {
    fn normal(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        match self {
            Base::Circle(c) => c.normal(t),
            Base::Squircle(s) => s.normal(t),
        }
    }
}

// Or maybe define a more refined Slice struct, allowing multiple interpolation strategies?
impl<F: Fn(f64) -> Coord> Curve2D for F {
    fn path(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        self(t)
    }
}

pub trait Texture {
    /// t ∈ \[0, 1\]
    fn f(&self, t: f64) -> f64;
}

struct TexturedCurve<C: DifferentiableCurve2D, T: Texture> {
    base: C,
    texture: T,
}

impl<C: DifferentiableCurve2D, T: Texture> Curve2D for TexturedCurve<C, T> {
    fn path(&self, t: f64) -> Coord {
        check_range(t, 0.0..=1.0);
        self.base.path(t) + self.base.normal(t) * self.texture.f(t)
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
        check_range(t, 0.0..=1.0);
        self.amplitude * (TAU * self.count as f64 * t).sin()
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

    /// u ∈ \[0, 1\]
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
        check_range(u, 0.0..=1.0);
        match &self.interpolation {
            InterpolationStrategy::Lerp(txt) => {
                let offset = txt.f(u);
                Box::new(move |t| {
                    let line = Line::new(self.bottom.path(t), self.top.path(t));
                    let p = line.point_at_ratio_from_start(&Euclidean, u).0;

                    // Calculate normal vectors from the bases of the textured curves
                    let normals = Line::new(self.bottom.base.normal(t), self.top.base.normal(t));
                    // Lerped normal vector at `u`
                    let n = normals.point_at_ratio_from_start(&Euclidean, u).0;
                    let Some(n) = n.try_normalize() else {
                        // when `self.bottom.base.normal(t) + self.top.base.normal(t) = (0, 0)`
                        panic!(
                            "The top and the bottom normal vectors point to the opposite directions at t = {t}, u = {u}: {n:?}"
                        )
                    };

                    p + n * offset
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
    /// ripple around the circumference
    flutes: SinusoidalRipples,
    /// ripple along the height
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

pub trait Lattice {
    /// Centers of cells overlapping with `window`
    fn centers(&self, window: Rect) -> MultiPoint; // Just Vec<Coord>?

    /// Corners of the "seed" cell at the origin (closed, CCW)
    fn seed(&self) -> LineString; // Or Polygon?
}

/// k's such that lo ≤ k * step ≤ hi, padded by one on each side
fn steps(lo: f64, hi: f64, step: f64) -> RangeInclusive<i64> {
    (((lo / step).floor() as i64) - 1)..=(((hi / step).ceil() as i64) + 1)
}

impl Lattice for Grid {
    fn centers(&self, window: Rect) -> MultiPoint {
        let s = self.spacing;
        let xs = steps(window.min().x, window.max().x, s);
        let ys = steps(window.min().y, window.max().y, s);

        let mut out = vec![];
        for j in ys {
            for i in xs.clone() {
                out.push(point!(x: i as f64, y: j as f64) * s);
            }
        }
        out.into()
    }

    fn seed(&self) -> LineString {
        let h = self.spacing / 2.0;
        line_string![
            (x: h, y: -h),
            (x: h, y: h),
            (x: -h, y: h),
            (x: -h, y: -h),
            (x: h, y: -h),
        ]
    }
}

impl Lattice for Honeycomb {
    fn centers(&self, window: Rect) -> MultiPoint {
        let s = self.spacing;
        // h = (3/2)r = (√3/2)s where r is the length of the edge (s/√3)
        let h = s * 3f64.sqrt() / 2.0;
        let xs = steps(window.min().x, window.max().x, s);
        let ys = steps(window.min().y, window.max().y, h);

        let mut out = vec![];
        for j in ys {
            let shift = if j % 2 != 0 { s / 2.0 } else { 0.0 };
            for i in xs.clone() {
                out.push(point!(x: i as f64 * s + shift, y: j as f64 * h));
            }
        }
        out.into()
    }

    fn seed(&self) -> LineString {
        use std::f64::consts::{FRAC_PI_3, FRAC_PI_6};
        // s/√3 (length of the edge)
        let r = self.spacing / 3f64.sqrt();
        let mut edges: LineString = (0..6)
            .map(|k| {
                let (s, c) = (FRAC_PI_6 + k as f64 * FRAC_PI_3).sin_cos();
                point!(x: c, y: s) * r
            })
            .collect();
        edges.close();
        edges
    }
}

impl Lattice for LatticePreset {
    fn centers(&self, window: Rect) -> MultiPoint {
        match self {
            LatticePreset::Grid(g) => g.centers(window),
            LatticePreset::Honeycomb(h) => h.centers(window),
        }
    }

    fn seed(&self) -> LineString {
        match self {
            LatticePreset::Grid(g) => g.seed(),
            LatticePreset::Honeycomb(h) => h.seed(),
        }
    }
}

/// Cell walls covering `window`
fn walls<L: Lattice>(lattice: &L, window: Rect) -> Vec<Line> {
    let seed = lattice.seed();
    debug_assert!(
        seed.is_closed() && seed.is_ccw(),
        "Seed should be closed and CCW"
    );

    // Only count a shared wall once
    let half: Vec<Line<f64>> = seed
        .lines()
        .filter(|line| {
            let (dx, dy) = line.delta().x_y();
            let eps = 1e-9 * dx.hypot(dy);
            // rightward || straight-up
            dx > eps || (dx.abs() <= eps && dy > 0.0)
        })
        .collect();

    lattice
        .centers(window)
        .iter()
        .flat_map(|c| half.iter().map(|edge| edge.translate(c.x(), c.y())))
        .collect()
}

/// Two circles around `center` with a polygon's boundary between them.
struct BoundaryBand {
    center: Coord,
    inner: f64,
    outer: f64,
}

impl BoundaryBand {
    fn new(polygon: &Polygon, center: Coord) -> Self {
        let boundary = polygon.exterior();
        let inner = if polygon.contains(&center) {
            Euclidean.distance(&Point(center), boundary)
        } else {
            0.0
        };
        let outer = boundary
            .coords()
            .map(|&c| Euclidean.distance(center, c))
            .fold(0.0, f64::max);
        BoundaryBand {
            center,
            inner,
            outer,
        }
    }

    /// In the inner disc, hence in the polygon
    fn encloses(&self, line: &Line) -> bool {
        // NOTE: use hand-rolled versions w/o sqrt if this is slow
        Euclidean.distance(self.center, line.start) < self.inner
            && Euclidean.distance(self.center, line.end) < self.inner
    }

    /// Past the outer circle, hence out of the polygon
    fn excludes(&self, line: &Line) -> bool {
        // NOTE: use hand-rolled versions w/o sqrt if this is slow
        Euclidean.distance(self.center, line) >= self.outer
    }
}

/// Lattice walls inside a closed curve, clipped against its sampled polygon
pub fn fill<L: Lattice>(lattice: &L, boundary: &dyn Curve2D) -> Vec<Line> {
    let mut sampled_boundary: LineString = (0..AROUND)
        .map(|i| boundary.path(i as f64 / AROUND as f64))
        .collect();
    sampled_boundary.close();

    let Some(window) = sampled_boundary.bounding_rect() else {
        return vec![];
    };

    let polygon = Polygon::new(sampled_boundary, vec![]);
    let band = BoundaryBand::new(&polygon, window.center());

    let (mut inside, undecided): (Vec<Line>, Vec<Line>) = walls(lattice, window)
        .iter()
        .partition(|wall| band.encloses(wall));
    let crossing: MultiLineString = undecided
        .iter()
        .filter(|wall| !band.excludes(wall))
        .collect();

    inside.extend(polygon.clip(&crossing, false).lines_iter());
    inside
}

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

pub type Path3D = Vec<[f32; 3]>;

/// Samples per slice
const AROUND: u32 = 256;
/// Samples from bottom to top
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
            for e in fill(lattice, &*sweep.slice(u0)) {
                mesh.wall(e.start, e.end, y0, y1);
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
    /// Push one slice at height u and return the index of the first vertex
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
            let Coord { x, y: mz } = curve.path(t);
            self.positions.extend([x as f32, y, -mz as f32]);
        }
        start
    }

    fn wall(&mut self, a: Coord<f64>, b: Coord<f64>, y0: f32, y1: f32) {
        let start = (self.positions.len() / 3) as u32;
        let ((ax, ay), (bx, by)) = (a.x_y(), b.x_y());
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

    /// Side wall from u0 to u1; outward = false faces it toward the axis
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

    /// A fan of triangles from the center of the slice at height u
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
    /// Push one slice at height u
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
            let Coord { x, y: mz } = curve.path(t);
            self.positions.push([x as f32, y, -mz as f32]);
        }
    }

    /// Side wall from u0 to u1
    fn side<S: Sweep>(&mut self, shape: &S, u0: f64, u1: f64) {
        (0..=UP).for_each(|j| self.slice(shape, u0 + (u1 - u0) * j as f64 / UP as f64));
    }
}

impl From<PathBuilder> for Path3D {
    fn from(PathBuilder { positions }: PathBuilder) -> Self {
        positions
    }
}
