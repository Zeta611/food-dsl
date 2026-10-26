import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import * as THREE from "three/webgpu";

// KEEP IN SYNC WITH `encodeParams` in Main.elm!
type Params = { radius: number; tube: number };

customElements.define(
  "render-preview",
  class extends HTMLElement {
    #params: Params = { radius: 10, tube: 3 };
    #torus?: THREE.Mesh;

    #applyParams() {
      if (!this.#torus) return;
      const { radius, tube } = this.#params;
      this.#torus.geometry.dispose();
      this.#torus.geometry = new THREE.TorusGeometry(radius, tube, 16, 100);
    }

    set params(p: Params) {
      this.#params = p;
      this.#applyParams();
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

      const material = new THREE.MeshStandardMaterial({ color: 0xff6347 });
      const torus = new THREE.Mesh(undefined, material);
      torus.position.y = 12;
      scene.add(torus);
      this.#torus = torus;
      this.#applyParams();

      const pointLight = new THREE.PointLight(0xffffff, 300);
      pointLight.position.set(10, 20, 10);
      scene.add(pointLight);

      const hemisphereLight = new THREE.HemisphereLight(0xffffff, 0x777777, 2);
      scene.add(hemisphereLight);

      const lightHelper = new THREE.PointLightHelper(pointLight, 0.8, 0x777711);

      const gridHelper = new THREE.GridHelper(200, 50);
      scene.add(lightHelper, gridHelper);

      const controls = new OrbitControls(camera, renderer.domElement);

      function animate(time: number) {
        torus.rotation.x = time / 2000;
        torus.rotation.y = time / 1000;
        torus.rotation.z = time / 2000;

        controls.update();
        renderer.render(scene, camera);
      }
      renderer.setAnimationLoop(animate);
    }
  },
);
