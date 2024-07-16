import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js';
import { PointerLockControls } from 'three/examples/jsm/controls/PointerLockControls.js';
import { Sky } from 'three/examples/jsm/objects/Sky.js';

let scene, camera, renderer, controls;
let raycaster = new THREE.Raycaster();
let mouse = new THREE.Vector2();
let selectedObject = null;
const regions = [], regionColors = new Map();
let minecraftWorld;

let moveForward = false;
let moveBackward = false;
let moveLeft = false;
let moveRight = false;
let moveUp = false;
let moveDown = false;
let canFly = true;
let collisionEnabled = true;

let prevTime = performance.now();
const velocity = new THREE.Vector3();
const direction = new THREE.Vector3();

let sky, sun;

document.addEventListener('DOMContentLoaded', () => {
  init();
  loadMinecraftWorld();
  animate();
});

function init() {
  setupScene();
  setupCamera();
  setupRenderer();
  setupSky();
  setupPointerLockControls();
  setupLighting();
  createAxes();
  setupEventListeners();
  updateWorldInfo();
}

function setupScene() {
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0x87CEEB); // Set the background color to sky blue
  scene.fog = new THREE.FogExp2(0x87CEEB, 0.00025); // Add fog with the same color
}

function setupCamera() {
  camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  camera.position.set(0, 50, 200);
}

function setupRenderer() {
  renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);
}

function setupSky() {
  sky = new Sky();
  sky.scale.setScalar(450000);
  scene.add(sky);

  sun = new THREE.Vector3();

  const effectController = {
    turbidity: 10,
    rayleigh: 3,
    mieCoefficient: 0.005,
    mieDirectionalG: 0.7,
    elevation: 2,
    azimuth: 180,
    exposure: renderer.toneMappingExposure
  };

  const uniforms = sky.material.uniforms;
  uniforms['turbidity'].value = effectController.turbidity;
  uniforms['rayleigh'].value = effectController.rayleigh;
  uniforms['mieCoefficient'].value = effectController.mieCoefficient;
  uniforms['mieDirectionalG'].value = effectController.mieDirectionalG;

  const phi = THREE.MathUtils.degToRad(90 - effectController.elevation);
  const theta = THREE.MathUtils.degToRad(effectController.azimuth);

  sun.setFromSphericalCoords(1, phi, theta);

  uniforms['sunPosition'].value.copy(sun);

  renderer.toneMappingExposure = effectController.exposure;
  renderer.render(scene, camera);
}

function setupPointerLockControls() {
  controls = new PointerLockControls(camera, document.body);

  const blocker = document.getElementById('blocker');
  const instructions = document.getElementById('instructions');

  instructions.addEventListener('click', function () {
    controls.lock();
  });

  controls.addEventListener('lock', function () {
    instructions.style.display = 'none';
    blocker.style.display = 'none';
  });

  controls.addEventListener('unlock', function () {
    blocker.style.display = 'block';
    instructions.style.display = '';
  });

  scene.add(controls.getObject());

  document.addEventListener('keydown', onKeyDown);
  document.addEventListener('keyup', onKeyUp);
}

function setupLighting() {
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
  const hemisphereLight = new THREE.HemisphereLight(0xffffbb, 0x080820, 1);
  directionalLight.position.set(1, 1, 1).normalize();
  scene.add(ambientLight, directionalLight, hemisphereLight);
}

function createAxes() {
  const material = new THREE.LineBasicMaterial({ color: 0xff0000 });
  const largeNumber = 10000;
  const axes = ['x', 'y', 'z'].map(axis => {
    const points = [new THREE.Vector3(), new THREE.Vector3()];
    points[0][axis] = -largeNumber;
    points[1][axis] = largeNumber;
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    return new THREE.Line(geometry, material);
  });
  scene.add(...axes);
}

function setupEventListeners() {
  window.addEventListener('resize', onWindowResize, false);
  window.addEventListener('mousemove', onMouseMove, false);
  window.addEventListener('click', onMouseClick, false);
  document.getElementById('clearSelectionButton').addEventListener('click', clearSelection);
  document.getElementById('addRegionButton').addEventListener('click', addRegion);
  document.getElementById('generateWorldConfigButton').addEventListener('click', generateWorldConfig);
  document.getElementById('cubeSizeInput').addEventListener('change', onCubeSizeChange);
  document.getElementById('loadConfigButton').addEventListener('change', loadConfig);
  document.getElementById('resetButton').addEventListener('click', resetCamera);
  document.addEventListener('keydown', (event) => {
    if (event.code === 'KeyR') {
      resetCamera();
    }
  });
}

function onCubeSizeChange(event) {
  cubeSize = parseInt(event.target.value);
  createCubesBasedOnMinecraftWorld();
}

function clearSelection() {
  regions.length = 0;
  if (selectedObject) {
    selectedObject.material.color.set(0xffffff); // Reset selection color
    selectedObject = null;
  }
  document.getElementById('regionList').innerHTML = '';
  document.getElementById('loadConfigButton').value = '';
  createCubesBasedOnMinecraftWorld();
}

function addRegion() {
  if (!selectedObject) return alert('No object selected!');
  const owner = prompt('Enter region owner:');
  if (!owner) return alert('Owner name is required!');
  const everyoneAllowed = confirm('Do you authorize all players to roam freely in your region? Click "OK" for Yes and "Cancel" for No.');
  let region = regions.find(r => r.owner === owner);
  const newCube = { center: [selectedObject.position.x, selectedObject.position.y, selectedObject.position.z], side_length: 16 };
  if (!regionColors.has(owner)) regionColors.set(owner, new THREE.Color(Math.random(), Math.random(), Math.random()));
  if (region) {
    const existingCenters = new Set(region.cubes.map(c => c.center.join(',')));
    if (!existingCenters.has(newCube.center.join(','))) {
      region.cubes.push(newCube);
    }
  } else {
    region = { owner, everyone_allowed: everyoneAllowed, authorized_players: [], cubes: [newCube] };
    regions.push(region);
  }
  selectedObject.material.color.set(0xffffff); // Reset selection color
  selectedObject = null;
  updateRegionList();
  colorRegionCubes(region);
}

function generateWorldConfig() {
  const json = JSON.stringify(regions, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'world_config.json';
  a.click();
  URL.revokeObjectURL(url);
}

function loadConfig(event) {
  const file = event.target.files[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = e => {
    const loadedRegions = JSON.parse(e.target.result);
    regions.length = 0;
    regions.push(...loadedRegions);
    updateRegionList();
  };
  reader.readAsText(file);
}

function colorRegionCubes(region) {
  const color = regionColors.get(region.owner) || new THREE.Color(Math.random(), Math.random(), Math.random());
  regionColors.set(region.owner, color);
  region.cubes.forEach(cubeData => {
    const cube = minecraftWorld.children.find(c => c.position.x === cubeData.center[0] && c.position.y === cubeData.center[1] && c.position.z === cubeData.center[2]);
    if (cube) {
      cube.material.color.set(color);
      cube.material.opacity = 0.2;
    }
  });
  updateWorldInfo();
}

function updateRegionList() {
  const regionList = document.getElementById('regionList');
  regionList.innerHTML = '';
  const ownerCubeCount = new Map();
  regions.forEach(region => {
    const cubeCount = region.cubes.length;
    ownerCubeCount.set(region.owner, (ownerCubeCount.get(region.owner) || 0) + cubeCount);
    if (!regionColors.has(region.owner)) regionColors.set(region.owner, new THREE.Color(Math.random(), Math.random(), Math.random()));
  });
  ownerCubeCount.forEach((cubeCount, owner) => {
    const color = regionColors.get(owner);
    const regionItem = document.createElement('div');
    regionItem.className = 'region-item';
    const colorBox = document.createElement('div');
    colorBox.className = 'region-color';
    colorBox.style.backgroundColor = color.getStyle();
    const ownerText = document.createElement('span');
    ownerText.textContent = `${owner} (${cubeCount} cubes)`;
    regionItem.appendChild(colorBox);
    regionItem.appendChild(ownerText);
    regionList.appendChild(regionItem);
  });
  updateWorldInfo();
}

function updateWorldInfo() {
  const totalCubes = minecraftWorld ? minecraftWorld.children.length : 0;
  const usedCubes = regions.reduce((sum, region) => sum + region.cubes.length, 0);
  const availableCubes = totalCubes - usedCubes;
  const worldSize = `${totalCubes} cubes`;
  document.getElementById('cubeCount').textContent = `Cubes Available: ${availableCubes}`;
  document.getElementById('worldSize').textContent = `World Size: ${worldSize}`;
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

function onMouseClick() {
  raycaster.setFromCamera(mouse, camera);
  const intersects = raycaster.intersectObject(minecraftWorld, true);
  if (intersects.length > 0) {
    const clickedObject = intersects[0].object;
    if (selectedObject !== clickedObject) {
      clearSelection();
      selectedObject = clickedObject;
      selectedObject.material.color.set(0xfd7904); // Highlight selected object
    } else {
      clearSelection();
    }

    // Log the position of the clicked object's center
    console.log(`Clicked object at position: (${clickedObject.position.x}, ${clickedObject.position.y}, ${clickedObject.position.z})`);
  }
}

function onKeyDown(event) {
  switch (event.code) {
    case 'ArrowUp':
    case 'KeyW':
      moveForward = true;
      break;
    case 'ArrowLeft':
    case 'KeyA':
      moveLeft = true;
      break;
    case 'ArrowDown':
    case 'KeyS':
      moveBackward = true;
      break;
    case 'ArrowRight':
    case 'KeyD':
      moveRight = true;
      break;
    case 'Space':
      if (canFly) moveUp = true;
      break;
    case 'ShiftLeft':
      if (canFly) moveDown = true;
      break;
    case 'KeyC':
      collisionEnabled = !collisionEnabled;
      break;
    case 'KeyF':
      canFly = !canFly;
      if (!canFly) {
        moveUp = false;
        moveDown = false;
      }
      break;
  }
}

function onKeyUp(event) {
  switch (event.code) {
    case 'ArrowUp':
    case 'KeyW':
      moveForward = false;
      break;
    case 'ArrowLeft':
    case 'KeyA':
      moveLeft = false;
      break;
    case 'ArrowDown':
    case 'KeyS':
      moveBackward = false;
      break;
    case 'ArrowRight':
    case 'KeyD':
      moveRight = false;
      break;
    case 'Space':
      moveUp = false;
      break;
    case 'ShiftLeft':
      moveDown = false;
      break;
  }
}

function loadMinecraftWorld() {
  const objLoader = new OBJLoader();
  const mtlLoader = new MTLLoader();
  const objPath = '/ExportedMinecraftWorld/minecraft.obj';
  const mtlPath = '/ExportedMinecraftWorld/minecraft.mtl';

  mtlLoader.load(
    mtlPath,
    (materials) => {
      materials.preload();
      objLoader.setMaterials(materials);
      objLoader.load(
        objPath,
        (object) => {
          object.traverse((child) => {
            if (child.isMesh) {
              child.userData.isMinecraftWorld = true;
              if (child.material.map) {
                const textureType = getTextureType(child.material.map.name);
                console.log(`Texture type: ${textureType}`);
              }
            }
          });
          scene.add(object);
          minecraftWorld = object;
          updateWorldInfo();
        },
        undefined,
        (error) => {
          console.error('Error loading OBJ file:', error);
        }
      );
    },
    undefined,
    (error) => {
      console.error('Error loading MTL file:', error);
    }
  );
}

function getTextureType(textureName) {
  if (textureName.includes('block')) return 'block';
  if (textureName.includes('entity')) return 'entity';
  if (textureName.includes('painting')) return 'painting';
  if (textureName.includes('banner')) return 'banner';
  if (textureName.includes('models')) return 'models';
  return 'block';
}

function resetCamera() {
  controls.getObject().position.set(0, 50, 200); // Adjust these values as needed
  velocity.set(0, 0, 0);
}

function animate() {
  requestAnimationFrame(animate);

  const time = performance.now();

  if (controls.isLocked === true) {
    const delta = (time - prevTime) / 1000;

    velocity.x -= velocity.x * 10.0 * delta;
    velocity.z -= velocity.z * 10.0 * delta;
    velocity.y -= velocity.y * 10.0 * delta; // Add damping to y axis

    direction.z = Number(moveForward) - Number(moveBackward);
    direction.x = Number(moveRight) - Number(moveLeft);
    direction.y = Number(moveUp) - Number(moveDown);
    direction.normalize();

    if (moveForward || moveBackward) velocity.z -= direction.z * 400.0 * delta;
    if (moveLeft || moveRight) velocity.x -= direction.x * 400.0 * delta;
    if (moveUp || moveDown) velocity.y += direction.y * 400.0 * delta;

    if (collisionEnabled) {
      // Collision detection
      const raycaster = new THREE.Raycaster();
      raycaster.set(controls.getObject().position, new THREE.Vector3(0, -1, 0));
      const intersects = raycaster.intersectObject(minecraftWorld, true);

      if (intersects.length > 0) {
        const distance = intersects[0].distance;
        if (distance < 10) {
          velocity.y = Math.max(0, velocity.y);
        }
      }
    }

    controls.moveRight(-velocity.x * delta);
    controls.moveForward(-velocity.z * delta);

    controls.getObject().position.y += velocity.y * delta;

    // If flying is toggled off, make the player drop down until they reach a cube
    if (!canFly && !collisionEnabled) {
      const raycaster = new THREE.Raycaster();
      raycaster.set(controls.getObject().position, new THREE.Vector3(0, -1, 0));
      const intersects = raycaster.intersectObject(minecraftWorld, true);

      if (intersects.length > 0) {
        const distance = intersects[0].distance;
        if (distance < 10) {
          velocity.y = Math.max(0, velocity.y);
        } else {
          velocity.y -= 9.8 * delta; // Apply gravity
        }
      } else {
        velocity.y -= 9.8 * delta; // Apply gravity
      }
    }

    // Keep the sky centered on the camera
    sky.position.copy(controls.getObject().position);
  }

  prevTime = time;

  renderer.render(scene, camera);
}

