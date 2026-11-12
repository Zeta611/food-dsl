import equal from "fast-deep-equal";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import * as THREE from "three/webgpu";
import init, {
  type Cup,
  infill_mesh,
  type LatticePreset,
  shell_mesh,
  shell_path,
} from "../../binding/pkg";
import {
  buildTracks,
  type Candidate,
  fanOut,
  getParam,
  type Param,
  projectOnTrack,
  type Track,
  withParam,
} from "./drag";

/** A translucent "what if" cup. `dim` fades it when another one is hovered. */
type Ghost = Candidate & { dim?: boolean };

/** Tracks whose distance to the pointer is within this many pixels of the
 * closest one count as "could also be meant": the drag is ambiguous. */
const AMBIGUITY_PX = 12;
/** How far off a track the pointer may stray, as a fraction of how far it has
 * moved (0.6 is about 37 degrees), before that track stops being a candidate. */
const OFF_TRACK = 0.6;
/** A parameter is offered only where it is nearly as influential as the most
 * influential one at the grabbed point. This is what makes the rim "own" the
 * top radius and the wall "own" the flutes and rings. */
const OWNERSHIP = 0.9;
/** Tracks shorter than this on screen are stretched to it. Flute and ring
 * depth move the surface by a couple of millimetres, far too little to drag
 * along, so the pointer gets a longer runway than the surface really travels. */
const MIN_TRACK_PX = 140;
/** How far from the grabbed point a track's name tag sits. */
const LABEL_PX = 110;
/** A press that moves less than this is a click, not a drag. */
const CLICK_PX = 4;

await init();

customElements.define(
  "render-preview",
  class extends HTMLElement {
    #cup?: Cup;
    #mesh?: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;

    #toolpath = false;
    #toolpathMesh?: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;

    #lattice: LatticePreset | null = null;
    #infillMesh?: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;

    // --- Direct manipulation ---
    #params: Param[] = [];
    /** Ghosts Elm asked for (shown whenever no drag is in progress) */
    #ghosts: Ghost[] = [];
    /** Everything drawn on top of the cup: ghosts, tracks, the grab point */
    #overlay = new THREE.Group();
    #camera?: THREE.PerspectiveCamera;
    /** Transient drag state. Elm never sees this, only the events it causes. */
    #drag?: {
      tracks: Track[];
      /** The point on the surface that was pressed */
      grabbed: THREE.Vector3;
      start: THREE.Vector2;
      pointer: THREE.Vector2;
      /** Pointer moved since the last frame, so the preview is stale */
      dirty: boolean;
    };

    /** Name tags for the tracks: plain HTML laid over the canvas */
    #labels: HTMLElement[] = [];

    #drawLabels(tracks: Track[], grabbed: THREE.Vector3, active: Set<string>) {
      const origin = this.#toScreen(grabbed);
      for (const { param, points } of tracks) {
        // A fixed distance along the track, toward the parameter's maximum
        const end = this.#toScreen(points[points.length - 1]);
        const at = origin.clone().add(end.sub(origin).setLength(LABEL_PX));
        const label = document.createElement("div");
        // "+" means: dropping here adds a slider to the panel
        label.textContent = param.hasControl ? param.label : `${param.label} +`;
        Object.assign(label.style, {
          position: "absolute",
          left: `${at.x}px`,
          top: `${at.y}px`,
          transform: "translate(-50%, -50%)",
          padding: "1px 8px",
          borderRadius: "999px",
          font: "12px system-ui, sans-serif",
          whiteSpace: "nowrap",
          color: "white",
          background: param.color,
          opacity: active.has(param.key) ? "1" : "0.45",
          pointerEvents: "none",
        });
        this.appendChild(label);
        this.#labels.push(label);
      }
    }

    #clearOverlay() {
      for (const label of this.#labels) label.remove();
      this.#labels = [];
      for (const child of [...this.#overlay.children]) {
        const mesh = child as THREE.Mesh<THREE.BufferGeometry, THREE.Material>;
        mesh.geometry.dispose();
        mesh.material.dispose();
        this.#overlay.remove(child);
      }
    }

    /** Draw on top of everything: an edit that shrinks the cup would
     * otherwise hide inside the opaque original. */
    #overlayMaterial(color: string, opacity: number) {
      return new THREE.MeshBasicMaterial({
        color,
        opacity,
        transparent: true,
        depthTest: false,
        depthWrite: false,
        side: THREE.DoubleSide,
      });
    }

    #drawGhosts(ghosts: Ghost[]) {
      if (!this.#cup) return;
      for (const ghost of ghosts) {
        const param = this.#params.find((p) => p.key === ghost.key);
        if (!param) continue;
        const mesh = shell_mesh(withParam(this.#cup, param, ghost.value));
        const { positions, indices } = mesh;
        mesh.free();
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
        geometry.setIndex(new THREE.BufferAttribute(indices, 1));
        const material = this.#overlayMaterial(param.color, ghost.dim ? 0.06 : 0.3);
        this.#overlay.add(new THREE.Mesh(geometry, material));
      }
    }

    #drawTracks(tracks: Track[], active: Set<string>) {
      for (const { param, points } of tracks) {
        const curve = new THREE.CatmullRomCurve3(points);
        const geometry = new THREE.TubeGeometry(curve, 16, 0.12, 6, false);
        const material = this.#overlayMaterial(param.color, active.has(param.key) ? 1 : 0.25);
        this.#overlay.add(new THREE.Mesh(geometry, material));
      }
    }

    #toScreen = (p: THREE.Vector3) => {
      const { clientWidth: w, clientHeight: h } = this;
      const ndc = p.clone().project(this.#camera as THREE.Camera);
      return new THREE.Vector2(((ndc.x + 1) / 2) * w, ((1 - ndc.y) / 2) * h);
    };

    #pointerOf(e: PointerEvent) {
      const rect = this.getBoundingClientRect();
      return new THREE.Vector2(e.clientX - rect.left, e.clientY - rect.top);
    }

    /** Which parameter changes does the current pointer position stand for? */
    #candidates(): Candidate[] {
      const drag = this.#drag;
      if (!drag || !this.#cup) return [];
      const cup = this.#cup;
      // A click (no real movement) proposes every parameter, unchanged:
      // "what can I control from here?"
      if (drag.pointer.distanceTo(drag.start) < CLICK_PX) {
        return drag.tracks.map((t) => ({ key: t.param.key, value: getParam(cup, t.param) }));
      }
      const hits = drag.tracks.map((t) => ({
        key: t.param.key,
        ...projectOnTrack(t, this.#toScreen, drag.pointer),
      }));
      // Only tracks the pointer is actually moving along. Dragging sideways,
      // across every track, proposes nothing instead of something arbitrary.
      const moved = drag.pointer.distanceTo(drag.start);
      const along = hits.filter((h) => h.distance <= OFF_TRACK * moved);
      const closest = Math.min(...along.map((h) => h.distance));
      return along
        .filter((h) => h.distance <= closest + AMBIGUITY_PX)
        .map(({ key, value }) => ({ key, value }));
    }

    #send(name: string, candidates: Candidate[] = []) {
      this.dispatchEvent(new CustomEvent(name, { detail: { candidates } }));
    }

    /** Called once per frame while dragging */
    #updateDrag() {
      const drag = this.#drag;
      if (!drag?.dirty) return;
      drag.dirty = false;
      const candidates = this.#candidates();
      this.#clearOverlay();
      this.#drawGhosts(candidates);
      const active = new Set(candidates.map((c) => c.key));
      this.#drawTracks(drag.tracks, active);
      this.#drawLabels(drag.tracks, drag.grabbed, active);
      this.#send("drag-preview", candidates);
    }

    #endDrag(event: "drag-end" | "drag-cancel") {
      if (!this.#drag) return;
      const candidates = event === "drag-end" ? this.#candidates() : [];
      this.#drag = undefined;
      this.#clearOverlay();
      this.#send(event, candidates);
    }

    /** Returns true if the press landed on the cup and started a drag */
    #startDrag(e: PointerEvent): boolean {
      if (!this.#mesh || !this.#cup || !this.#camera || e.button !== 0) return false;
      const pointer = this.#pointerOf(e);
      const ndc = new THREE.Vector2(
        (pointer.x / this.clientWidth) * 2 - 1,
        1 - (pointer.y / this.clientHeight) * 2,
      );
      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(ndc, this.#camera);
      const hit = raycaster.intersectObject(this.#mesh)[0];
      if (!hit?.face) return false;

      // Grab the corner of the hit triangle nearest to the hit point
      const position = this.#mesh.geometry.getAttribute("position");
      const corner = (i: number) => new THREE.Vector3().fromBufferAttribute(position, i);
      const { a, b, c } = hit.face;
      const vertex = [a, b, c].reduce((best, i) =>
        corner(i).distanceTo(hit.point) < corner(best).distanceTo(hit.point) ? i : best,
      );

      const all = buildTracks(this.#cup, this.#params, vertex);
      if (all.length === 0) return false;
      const strongest = Math.max(...all.map((t) => t.influence));
      const grabbed = corner(vertex);
      const stretched = all
        .filter((t) => t.influence >= OWNERSHIP * strongest)
        .map((t) => {
          // Stretch a short track about the grabbed point
          const [first, last] = [t.points[0], t.points[t.points.length - 1]];
          const pixels = this.#toScreen(first).distanceTo(this.#toScreen(last));
          const gain = Math.max(1, MIN_TRACK_PX / pixels);
          const points = t.points.map((p) =>
            grabbed.clone().addScaledVector(p.clone().sub(grabbed), gain),
          );
          return { ...t, points };
        });
      const viewAxis = grabbed.clone().sub(this.#camera.position).normalize();
      const tracks = fanOut(stretched, grabbed, viewAxis, this.#toScreen);
      this.#drag = { tracks, grabbed, start: pointer, pointer, dirty: true };
      return true;
    }

    #applyCup() {
      if (!this.#mesh || !this.#cup) return;

      const mesh = shell_mesh(this.#cup);
      const { positions, indices } = mesh;
      mesh.free();

      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      geometry.setIndex(new THREE.BufferAttribute(indices, 1));
      // Rust only sends positions + indices, so calculate normals
      geometry.computeVertexNormals();
      this.#mesh.geometry.dispose();
      this.#mesh.geometry = geometry;
    }

    #updateToolpath() {
      if (!this.#toolpathMesh) return;
      this.#toolpathMesh.visible = this.#toolpath;

      if (!this.#cup || !this.#toolpath) return;
      const positions = shell_path(this.#cup);

      const points = Array.from({ length: positions.length / 3 }, (_, i) =>
        new THREE.Vector3().fromArray(positions, 3 * i),
      );
      const curve = new THREE.CatmullRomCurve3(points);
      const geometry = new THREE.TubeGeometry(curve, points.length, 0.3, 8, false);
      this.#toolpathMesh.geometry.dispose();
      this.#toolpathMesh.geometry = geometry;
    }

    #updateInfill() {
      if (!this.#infillMesh) return;
      this.#infillMesh.visible = this.#lattice !== null;

      if (!this.#cup || !this.#lattice) return;
      const mesh = infill_mesh(this.#cup, this.#lattice);
      const { positions, indices } = mesh;
      mesh.free();

      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      geometry.setIndex(new THREE.BufferAttribute(indices, 1));
      geometry.computeVertexNormals();
      this.#infillMesh.geometry.dispose();
      this.#infillMesh.geometry = geometry;
    }

    // Elm sets these via `property "cup" ...`, `property "toolpath" ...`, etc.
    // Elm may set them before the mesh exists, so store them first.
    set cup(c: Cup) {
      if (equal(this.#cup, c)) {
        return;
      }
      this.#cup = c;
      this.#applyCup();
      this.#updateToolpath();
      this.#updateInfill();
    }

    set toolpath(w: boolean) {
      this.#toolpath = w;
      this.#updateToolpath();
    }

    set lattice(l: LatticePreset | null) {
      if (equal(this.#lattice, l)) {
        return;
      }
      this.#lattice = l;
      this.#updateInfill();
    }

    set params(p: Param[]) {
      this.#params = p;
    }

    set ghosts(g: Ghost[]) {
      if (equal(this.#ghosts, g)) return;
      this.#ghosts = g;
      // While dragging, the element draws its own previews
      if (this.#drag) return;
      this.#clearOverlay();
      this.#drawGhosts(g);
    }

    connectedCallback() {
      this.init();
    }

    async init() {
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
      this.#camera = camera;

      camera.position.x = 30;
      camera.position.y = 40;
      camera.position.z = 20;

      const canvas = document.createElement("canvas");
      this.appendChild(canvas);

      const renderer = new THREE.WebGPURenderer({ canvas, antialias: true });
      await renderer.init();

      new ResizeObserver(() => {
        const { clientWidth: w, clientHeight: h } = this;
        renderer.setSize(w, h);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
      }).observe(this);

      renderer.toneMapping = THREE.AgXToneMapping;
      renderer.shadowMap.enabled = true;

      const pmrem = new THREE.PMREMGenerator(renderer);
      scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
      scene.environmentIntensity = 0.5;

      const material = new THREE.MeshPhysicalMaterial({
        color: 0x3b1e10,
        roughness: 0.4,
        clearcoat: 0.2,
        clearcoatRoughness: 0.3,
        side: THREE.DoubleSide,
      });
      this.#mesh = new THREE.Mesh(new THREE.BufferGeometry(), material);
      this.#mesh.castShadow = true;
      scene.add(this.#mesh);
      this.#applyCup();

      this.#toolpathMesh = new THREE.Mesh(new THREE.BufferGeometry(), material);
      scene.add(this.#toolpathMesh);
      this.#updateToolpath();

      this.#infillMesh = new THREE.Mesh(new THREE.BufferGeometry(), material);
      this.#infillMesh.castShadow = true;
      scene.add(this.#infillMesh);
      this.#updateInfill();

      const keyLight = new THREE.DirectionalLight(0xfff1e0, 4.5);
      keyLight.position.set(40, 80, 30);
      keyLight.castShadow = true;
      keyLight.shadow.mapSize.set(2048, 2048);
      keyLight.shadow.camera.left = -60;
      keyLight.shadow.camera.right = 60;
      keyLight.shadow.camera.top = 60;
      keyLight.shadow.camera.bottom = -60;
      keyLight.shadow.normalBias = 0.05;
      keyLight.shadow.radius = 4;
      scene.add(keyLight);

      const floor = new THREE.Mesh(
        new THREE.PlaneGeometry(400, 400),
        new THREE.ShadowMaterial({ opacity: 0.25 }),
      );
      floor.rotation.x = -Math.PI / 2;
      floor.receiveShadow = true;
      scene.add(floor);

      const gridHelper = new THREE.GridHelper(200, 20, 0xc93400, 0xffd6a7);
      gridHelper.material.toneMapped = false;
      scene.add(gridHelper);

      scene.add(this.#overlay);
      this.#drawGhosts(this.#ghosts);

      // Registered before OrbitControls so that we can switch it off in time:
      // pressing the cup edits it, pressing empty space orbits the camera.
      canvas.addEventListener("pointerdown", (e) => {
        if (!this.#startDrag(e)) return;
        controls.enabled = false;
        canvas.setPointerCapture(e.pointerId);
      });
      canvas.addEventListener("pointermove", (e) => {
        if (!this.#drag) return;
        this.#drag.pointer = this.#pointerOf(e);
        this.#drag.dirty = true;
      });
      const finish = (event: "drag-end" | "drag-cancel") => {
        this.#endDrag(event);
        controls.enabled = true;
        // Elm may have set ghosts while we were dragging
        this.#drawGhosts(this.#ghosts);
      };
      canvas.addEventListener("pointerup", () => this.#drag && finish("drag-end"));
      canvas.addEventListener("pointercancel", () => this.#drag && finish("drag-cancel"));
      window.addEventListener("keydown", (e) => {
        if (e.key === "Escape" && this.#drag) finish("drag-cancel");
      });

      const controls = new OrbitControls(camera, renderer.domElement);

      const animate = () => {
        this.#updateDrag();
        controls.update();
        renderer.render(scene, camera);
      };
      renderer.setAnimationLoop(animate);
    }
  },
);
