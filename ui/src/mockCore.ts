// Mockup before the Rust core impl. Turns a high-level program into triangle buffers

/** KEEP IN SYNC WITH `encodeCup` in Main.elm! */
export type Cup = {
  bottomRadius: number;
  topRadius: number;
  height: number;
  /** ripple around the circumference */
  flutes: { amplitude: number; count: number };
  /** ripple along the height */
  rings: { amplitude: number; count: number };
};

/** Samples per ring */
const AROUND = 256;
/** Samples from bottom to top */
const UP = 64;

/** Radius at height u ∈ [0, 1] and angle t ∈ [0, 1). */
function radius(cup: Cup, u: number, t: number): number {
  const base = cup.bottomRadius + (cup.topRadius - cup.bottomRadius) * u;
  const flute = cup.flutes.amplitude * Math.sin(2 * Math.PI * cup.flutes.count * t);
  const ring = cup.rings.amplitude * Math.sin(2 * Math.PI * cup.rings.count * u);
  return base + flute + ring;
}

export function compile(cup: Cup): { positions: Float32Array; indices: Uint32Array } {
  const positions: number[] = [];
  const indices: number[] = [];

  // Push one ring at height u and return the index of the first vertex
  const ring = (u: number): number => {
    const start = positions.length / 3;
    for (let i = 0; i < AROUND; ++i) {
      const t = i / AROUND;
      const r = radius(cup, u, t);
      const angle = 2 * Math.PI * t;
      positions.push(r * Math.cos(angle), cup.height * u, r * Math.sin(angle));
    }
    return start;
  };

  // Side crust: two triangles per grid cell between rings j and j + 1
  const rows: number[] = [];
  for (let j = 0; j <= UP; ++j) {
    rows.push(ring(j / UP));
  }
  for (let j = 0; j < UP; ++j) {
    for (let i = 0; i < AROUND; ++i) {
      const next = (i + 1) % AROUND;
      const a = rows[j] + i;
      const b = rows[j + 1] + i;
      const c = rows[j] + next;
      const d = rows[j + 1] + next;
      // CCW means front
      indices.push(a, b, c, c, b, d);
    }
  }

  // Caps: a fan of triangles from the center
  const cap = (u: number) => {
    const center = positions.length / 3;
    positions.push(0, cup.height * u, 0);
    const rim = ring(u);
    for (let i = 0; i < AROUND; ++i) {
      const p = rim + i;
      const q = rim + ((i + 1) % AROUND);
      if (u === 1) {
        indices.push(center, q, p);
      } else {
        indices.push(center, p, q);
      }
    }
  };
  cap(0);
  cap(1);

  return { positions: new Float32Array(positions), indices: new Uint32Array(indices) };
}
