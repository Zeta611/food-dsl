import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import * as THREE from "three/webgpu";
import { type Cup, compile } from "./mockCore";

customElements.define(
  "render-preview",
  class extends HTMLElement {
    #cup?: Cup;
    #wireframe = false;
    #mesh?: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;

    #applyCup() {
      if (!this.#mesh || !this.#cup) return;

      const { positions, indices } = compile(this.#cup);
      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      geometry.setIndex(new THREE.BufferAttribute(indices, 1));
      geometry.computeVertexNormals();
      this.#mesh.geometry.dispose();
      this.#mesh.geometry = geometry;
    }

    // Elm sets these via `property "cup" ...` and `property "wireframe" ...`.
    // Elm may set them before the mesh exists, so store them first.
    set cup(c: Cup) {
      this.#cup = c;
      this.#applyCup();
    }

    set wireframe(w: boolean) {
      this.#wireframe = w;
      if (this.#mesh) this.#mesh.material.wireframe = w;
    }

    connectedCallback() {
      this.init();
    }

    init() {
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);

      camera.position.x = 30;
      camera.position.y = 40;
      camera.position.z = 20;

      const canvas = document.createElement("canvas");
      this.appendChild(canvas);

      const renderer = new THREE.WebGPURenderer({ canvas });

      new ResizeObserver(() => {
        const { clientWidth: w, clientHeight: h } = this;
        renderer.setSize(w, h);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
      }).observe(this);

      const material = new THREE.MeshStandardMaterial({
        color: 0x7b3f00,
        wireframe: this.#wireframe,
      });
      this.#mesh = new THREE.Mesh(new THREE.BufferGeometry(), material);
      scene.add(this.#mesh);
      this.#applyCup();

      const pointLight = new THREE.PointLight(0xffffff, 300);
      pointLight.position.set(10, 20, 10);
      scene.add(pointLight);

      const hemisphereLight = new THREE.HemisphereLight(0xffffff, 0x777777, 2);
      scene.add(hemisphereLight);

      const lightHelper = new THREE.PointLightHelper(pointLight, 0.8, 0x777711);

      const gridHelper = new THREE.GridHelper(200, 50);
      scene.add(lightHelper, gridHelper);

      const controls = new OrbitControls(camera, renderer.domElement);

      function animate() {
        controls.update();
        renderer.render(scene, camera);
      }
      renderer.setAnimationLoop(animate);
    }
  },
);
