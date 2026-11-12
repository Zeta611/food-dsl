// Model-driven dragging for the cup preview.
//
// The idea (after "Declarative Dragging", Horowitz et al. 2026): we never ask
// which parameter a point on the surface "belongs to". Instead we re-render the
// cup with one parameter changed and look at where the grabbed vertex ended up.
// That works because the core always samples the cup on the same grid, so
// vertex i is the same place on the surface in every rendering.

import * as THREE from "three/webgpu";
import { type Cup, shell_mesh } from "../../binding/pkg";

/** One adjustable number in the cup. The table itself lives in Main.elm. */
export type Param = {
  key: string;
  label: string;
  /** Whether the panel already has a slider for this parameter */
  hasControl: boolean;
  /** Where the number sits in the cup JSON, e.g. ["flutes", "amplitude"] */
  path: string[];
  min: number;
  max: number;
  step: number;
  color: string;
};

/** "Set this parameter to this value": what a drag proposes to Elm. */
export type Candidate = { key: string; value: number };

type Json = Record<string, unknown>;

export function getParam(cup: Cup, param: Param): number {
  let node: unknown = cup;
  for (const k of param.path) node = (node as Json)[k];
  return node as number;
}

export function withParam(cup: Cup, param: Param, value: number): Cup {
  const copy = structuredClone(cup);
  let node = copy as unknown as Json;
  for (const k of param.path.slice(0, -1)) node = node[k] as Json;
  node[param.path[param.path.length - 1]] = value;
  return copy;
}

/** The speculative render: only the vertex positions, nothing drawn. */
function positionsOf(cup: Cup): Float32Array {
  const mesh = shell_mesh(cup);
  const { positions } = mesh;
  mesh.free();
  return positions;
}

/** The path the grabbed vertex travels as one parameter sweeps its range. */
export type Track = {
  param: Param;
  values: number[];
  points: THREE.Vector3[];
  /**
   * How much this parameter moves the grabbed vertex, relative to the vertex
   * it moves the most (0 to 1). Top radius scores 1 at the rim and 0.5 halfway
   * up the wall; flute depth scores 1 on a crest and 0 between crests.
   */
  influence: number;
};

const SAMPLES = 5;
/** A parameter that moves the vertex less than this (mm) is not a candidate. */
const MIN_TRAVEL = 0.05;

export function buildTracks(cup: Cup, params: Param[], vertex: number): Track[] {
  const tracks: Track[] = [];
  for (const param of params) {
    const values = Array.from(
      { length: SAMPLES },
      (_, i) => param.min + ((param.max - param.min) * i) / (SAMPLES - 1),
    );
    const renders = values.map((v) => positionsOf(withParam(cup, param, v)));
    const points = renders.map((r) => new THREE.Vector3().fromArray(r, 3 * vertex));
    const travel = points[0].distanceTo(points[SAMPLES - 1]);
    if (travel <= MIN_TRAVEL) continue;

    // The farthest any vertex travels between the two ends of the range
    const [first, last] = [renders[0], renders[SAMPLES - 1]];
    let farthest = 0;
    for (let i = 0; i < first.length; i += 3) {
      const d = Math.hypot(
        last[i] - first[i],
        last[i + 1] - first[i + 1],
        last[i + 2] - first[i + 2],
      );
      farthest = Math.max(farthest, d);
    }
    tracks.push({ param, values, points, influence: travel / farthest });
  }
  return tracks;
}

/**
 * Project the pointer onto a track, both in screen pixels.
 * Returns how far the pointer is from the track and which parameter value
 * the nearest point on the track stands for.
 */
export function projectOnTrack(
  track: Track,
  toScreen: (p: THREE.Vector3) => THREE.Vector2,
  pointer: THREE.Vector2,
): { distance: number; value: number } {
  const screen = track.points.map(toScreen);
  let best = { distance: Number.POSITIVE_INFINITY, value: track.values[0] };
  for (let i = 0; i + 1 < screen.length; i++) {
    const [a, b] = [screen[i], screen[i + 1]];
    const ab = b.clone().sub(a);
    const len2 = ab.lengthSq();
    // t = how far along the segment a→b the pointer falls, clamped to the segment
    const t = len2 === 0 ? 0 : THREE.MathUtils.clamp(pointer.clone().sub(a).dot(ab) / len2, 0, 1);
    const distance = a.clone().addScaledVector(ab, t).distanceTo(pointer);
    if (distance < best.distance) {
      const raw = track.values[i] + (track.values[i + 1] - track.values[i]) * t;
      const { min, max, step } = track.param;
      const snapped = Math.round(raw / step) * step;
      // toFixed kills float noise like 0.30000000000000004
      best = { distance, value: Number(THREE.MathUtils.clamp(snapped, min, max).toFixed(6)) };
    }
  }
  return best;
}

/** Tracks whose on-screen directions are closer than this count as overlapping. */
const OVERLAP = THREE.MathUtils.degToRad(15);
/** The angle put between overlapping tracks. */
const FAN = THREE.MathUtils.degToRad(30);

/**
 * Top radius, flute depth and ring depth all push a wall point straight
 * outward, so their tracks land on top of each other and the pointer cannot
 * choose between them. Spread such tracks into a fan around the grabbed point.
 * A fanned track is no longer where the surface really goes; it is a direction
 * to steer toward.
 */
export function fanOut(
  tracks: Track[],
  grabbed: THREE.Vector3,
  /** Unit vector from the camera to the grabbed point: the fan turns about it */
  viewAxis: THREE.Vector3,
  toScreen: (p: THREE.Vector3) => THREE.Vector2,
): Track[] {
  // A track is a line through the grabbed point, so its direction is an
  // angle between 0 and 180 degrees: pointing left is the same as right.
  const angleOf = ({ points }: Track) => {
    const d = toScreen(points[points.length - 1]).sub(toScreen(points[0]));
    return (Math.atan2(d.y, d.x) + Math.PI) % Math.PI;
  };
  const apart = (a: number, b: number) => Math.min(Math.abs(a - b), Math.PI - Math.abs(a - b));

  const groups: { angle: number; tracks: Track[] }[] = [];
  for (const track of tracks) {
    const angle = angleOf(track);
    const group = groups.find((g) => apart(g.angle, angle) < OVERLAP);
    if (group) group.tracks.push(track);
    else groups.push({ angle, tracks: [track] });
  }

  return groups.flatMap((group) =>
    group.tracks.map((track, i) => {
      // -1, 0, +1 fan steps for three tracks; -0.5, +0.5 for two; 0 for one
      const turn = (i - (group.tracks.length - 1) / 2) * FAN;
      const points = track.points.map((p) =>
        grabbed.clone().add(p.clone().sub(grabbed).applyAxisAngle(viewAxis, turn)),
      );
      return { ...track, points };
    }),
  );
}
