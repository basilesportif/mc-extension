import * as THREE from "three";
import { Sky } from "three/examples/jsm/objects/Sky.js";

const setupScene = (sceneRef) => {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x87ceeb);
  scene.fog = new THREE.FogExp2(0x87ceeb, 0.00025);
  sceneRef.current = scene;
  console.log("Scene set up.");
};

const setupCamera = (cameraRef, containerRef) => {
  if (containerRef.current) {
    const width = containerRef.current.clientWidth;
    const height = containerRef.current.clientHeight;
    const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
    camera.position.set(0, 50, 200);
    cameraRef.current = camera;
    console.log("Camera set up.");
  }
};

const setupRenderer = (rendererRef, containerRef) => {
  if (!rendererRef.current && containerRef.current) {
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    const width = containerRef.current.clientWidth;
    const height = containerRef.current.clientHeight;
    renderer.setSize(width, height);
    renderer.setPixelRatio(window.devicePixelRatio);
    rendererRef.current = renderer;
    console.log("Renderer created:", renderer);
    containerRef.current.appendChild(renderer.domElement);
    console.log("Renderer set up.");
  }
};

const setupSky = (sceneRef, skyRef, rendererRef) => {
  const sky = new Sky();
  sky.scale.setScalar(450000);
  sceneRef.current.add(sky);
  skyRef.current = sky;

  const sun = new THREE.Vector3();
  const effectController = {
    turbidity: 10,
    rayleigh: 2,
    mieCoefficient: 0.005,
    mieDirectionalG: 0.8,
    elevation: 2,
    azimuth: 180,
    exposure: rendererRef.current.toneMappingExposure,
  };
  const uniforms = sky.material.uniforms;
  uniforms["turbidity"].value = effectController.turbidity;
  uniforms["rayleigh"].value = effectController.rayleigh;
  uniforms["mieCoefficient"].value = effectController.mieCoefficient;
  uniforms["mieDirectionalG"].value = effectController.mieDirectionalG;

  const phi = THREE.MathUtils.degToRad(90 - effectController.elevation);
  const theta = THREE.MathUtils.degToRad(effectController.azimuth);

  sun.setFromSphericalCoords(1, phi, theta);

  uniforms["sunPosition"].value.copy(sun);
  console.log("Sky set up.");
};

const setupLighting = (sceneRef) => {
  // Ambient light
  const ambientLight = new THREE.AmbientLight(0x404040, 0.8); // Increase ambient light intensity

  // Directional light (sun-like)
  const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2); // Increase directional light intensity
  directionalLight.position.set(200, 200, 100); // Adjust position for better lighting
  directionalLight.castShadow = true;

  // Shadow map configuration
  directionalLight.shadow.mapSize.width = 2048; // Increase shadow map resolution
  directionalLight.shadow.mapSize.height = 2048;
  directionalLight.shadow.camera.near = 1; // Adjust near and far planes
  directionalLight.shadow.camera.far = 500;

  sceneRef.current.add(ambientLight, directionalLight);

  console.log("Lighting set up.");
};

const createAxes = (sceneRef) => {
  const material = new THREE.LineBasicMaterial({ color: 0xff0000 });
  const largeNumber = 10000;
  const axes = ["x", "y", "z"].map((axis) => {
    const points = [new THREE.Vector3(), new THREE.Vector3()];
    points[0][axis] = -largeNumber;
    points[1][axis] = largeNumber;
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    return new THREE.Line(geometry, material);
  });
  sceneRef.current.add(...axes);
  console.log("Axes created.");
};

const setupThreeJsScene = (sceneRef, cameraRef, rendererRef, containerRef, skyRef) => {
  setupScene(sceneRef);
  setupCamera(cameraRef, containerRef);
  setupRenderer(rendererRef, containerRef);
  setupSky(sceneRef, skyRef, rendererRef);
  setupLighting(sceneRef);
  createAxes(sceneRef);
};

export { setupThreeJsScene, setupScene, setupCamera, setupRenderer, setupSky, setupLighting, createAxes };