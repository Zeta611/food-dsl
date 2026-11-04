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
      const geometry = new THREE.TubeGeometry(curve, points.length, 0.3, 8, true);
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
      this.#lattice = l;
      this.#updateInfill();
    }

    connectedCallback() {
      this.init();
    }

    async init() {
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);

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

      const controls = new OrbitControls(camera, renderer.domElement);

      function animate() {
        controls.update();
        renderer.render(scene, camera);
      }
      renderer.setAnimationLoop(animate);
    }
  },
);
