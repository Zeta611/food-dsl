import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import * as THREE from "three/webgpu";

customElements.define(
  "render-controller",
  class extends HTMLElement {
    connectedCallback() {
      this.init();
    }

    init() {
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(
        75,
        window.innerWidth / window.innerHeight,
        0.1,
        1000,
      );

      camera.position.x = 30;
      camera.position.y = 40;
      camera.position.z = 20;

      const canvas = document.querySelector<HTMLCanvasElement>("canvas#preview");
      if (!canvas) {
        throw new Error("Could not find canvas#preview");
      }

      const renderer = new THREE.WebGPURenderer({ canvas });
      renderer.setSize(window.innerWidth, window.innerHeight);

      const geometry = new THREE.TorusGeometry(10, 3, 16, 100);
      const material = new THREE.MeshStandardMaterial({ color: 0xff6347 });
      const torus = new THREE.Mesh(geometry, material);
      torus.position.y = 12;
      scene.add(torus);

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
