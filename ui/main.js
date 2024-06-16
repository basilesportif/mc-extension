import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

let scene, camera, renderer, controls;
let raycaster = new THREE.Raycaster();
let mouse = new THREE.Vector2();
let INTERSECTED;
let selectedCube = null;
// Grid of cubes
const gridSize = 10;
const height = 4;
const cubeSize = 10;
const cubes = [];

init();
animate();

function init() {
  // Scene
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xffffff);

  // Camera
  camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  camera.position.set(0, 50, 200);

  // Renderer
  renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  // Controls
  controls = new OrbitControls(camera, renderer.domElement);

  // Lighting
  const ambientLight = new THREE.AmbientLight(0x404040);
  scene.add(ambientLight);
  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
  directionalLight.position.set(1, 1, 1).normalize();
  scene.add(directionalLight);

  for (let i = 0; i < gridSize; i++) {
    for (let j = 0; j < gridSize; j++) {
      for (let k = 0; k < height; k++) {
        const geometry = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
        const material = new THREE.MeshStandardMaterial({
          color: 0x000000,
          transparent: true,
          opacity: 0.1
        });
        const cube = new THREE.Mesh(geometry, material);
        cube.position.set(i * (cubeSize + 1) - (gridSize * (cubeSize + 1)) / 2, k * (cubeSize + 1), j * (cubeSize + 1) - (gridSize * (cubeSize + 1)) / 2);
        cube.userData.clicked = false;  // Track if the cube has been clicked
        scene.add(cube);
        cubes.push(cube);
      }
    }
  }

  const axesHelper = new THREE.AxesHelper(200);
  axesHelper.setColors(0xffffff, 0xffffff, 0xffffff);
  scene.add(axesHelper);


  // Event listeners
  window.addEventListener('resize', onWindowResize, false);
  window.addEventListener('mousemove', onMouseMove, false);
  window.addEventListener('click', onMouseClick, false);
  window.addEventListener('keydown', onKeyDown, false);
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
}

function onMouseMove(event) {
  mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
  mouse.y = - (event.clientY / window.innerHeight) * 2 + 1;
}

function onMouseClick(event) {
  raycaster.setFromCamera(mouse, camera);
  const intersects = raycaster.intersectObjects(scene.children);

  if (intersects.length > 0) {
    const clickedObject = intersects[0].object;
    if (!clickedObject.userData.clicked) {
      clickedObject.material.color.set(0xff0000);
      clickedObject.material.opacity = 0.7;
      clickedObject.userData.clicked = true;
      console.log('Selected & clicked cube position:', clickedObject.position);
      setSelectedCube(clickedObject);
    }
    else {
      clickedObject.material.color.set(0x000000);
      clickedObject.material.opacity = 0.1;
      clickedObject.userData.clicked = false;  // Mark the cube as clicked
    }
  }
}

function onKeyDown(event) {
  if (selectedCube) {
    let x = selectedCube.userData.gridX;
    let z = selectedCube.userData.gridZ;

    switch (event.key) {
      case 'ArrowUp':
        if (z > 0) z--;
        break;
      case 'ArrowDown':
        if (z < gridSize - 1) z++;
        break;
      case 'ArrowLeft':
        if (x > 0) x--;
        break;
      case 'ArrowRight':
        if (x < gridSize - 1) x++;
        break;
      default:
        return;
    }
    const nextCube = cubes.find(cube => cube.userData.gridX === x && cube.userData.gridZ === z);
    console.log(nextCube)
    if (nextCube) {
      setSelectedCube(nextCube);
    }
  }
}

function setSelectedCube(cube) {
  if (selectedCube) {
    if (!selectedCube.userData.clicked) {
      selectedCube.material.color.set(0x00ff00);  // Reset color if not clicked
    }
  }
  selectedCube = cube;
  if (!selectedCube.userData.clicked) {
    selectedCube.material.color.set(0xff0000);  // Set color to red
  }
}

function animate() {
  requestAnimationFrame(animate);

  // Raycasting
  raycaster.setFromCamera(mouse, camera);

  const intersects = raycaster.intersectObjects(scene.children);
  if (intersects.length > 0) {
    if (INTERSECTED != intersects[0].object) {
      if (INTERSECTED && !INTERSECTED.userData.clicked) {
        INTERSECTED.material.color.set(0x000000);  // Reset color if not clicked
      }
      INTERSECTED = intersects[0].object;
      if (!INTERSECTED.userData.clicked) {
        INTERSECTED.material.color.set(0xffff00);
      }
    }
  } else {
    if (INTERSECTED && !INTERSECTED.userData.clicked) {
      INTERSECTED.material.color.set(0x000000);  // Reset color if not clicked
    }
    INTERSECTED = null;
  }

  controls.update();
  renderer.render(scene, camera);
}
