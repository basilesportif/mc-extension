import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js';

let scene, camera, renderer, controls;
let raycaster = new THREE.Raycaster();
let mouse = new THREE.Vector2();
let selectedCube = null;
const cubes = [], regions = [], regionColors = new Map();
let minecraftWorld;

document.addEventListener('DOMContentLoaded', () => {
  init();
  loadMinecraftWorld();
  animate();
});

function init() {
  setupScene();
  setupCamera();
  setupRenderer();
  setupControls();
  setupLighting();
  createAxes();
  setupEventListeners();
  updateWorldInfo();
}

function setupScene() {
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xffffff);
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

function setupControls() {
  controls = new OrbitControls(camera, renderer.domElement);
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
  window.addEventListener('keydown', onKeyDown, false);
  document.getElementById('clearSelectionButton').addEventListener('click', clearSelection);
  document.getElementById('addRegionButton').addEventListener('click', addRegion);
  document.getElementById('generateWorldConfigButton').addEventListener('click', generateWorldConfig);
  document.getElementById('cubeSizeInput').addEventListener('change', onCubeSizeChange);
  document.getElementById('loadConfigButton').addEventListener('change', loadConfig);
}

function onCubeSizeChange(event) {
  cubeSize = parseInt(event.target.value);
  createCubesBasedOnMinecraftWorld();
}

function clearSelection() {
  regions.length = 0;
  cubes.forEach(cube => {
    cube.userData.clicked = false;
    cube.material.color.set(0x000000);
    cube.material.opacity = 0.05;
  });
  selectedCube = null;
  document.getElementById('regionList').innerHTML = '';
  document.getElementById('loadConfigButton').value = '';
  createCubesBasedOnMinecraftWorld();
}

function addRegion() {
  const clickedCubes = cubes.filter(cube => cube.userData.clicked);
  if (clickedCubes.length === 0) return alert('No cubes selected!');
  const owner = prompt('Enter region owner:');
  if (!owner) return alert('Owner name is required!');
  const everyoneAllowed = confirm('Do you authorize all players to roam freely in your region? Click "OK" for Yes and "Cancel" for No.');
  let region = regions.find(r => r.owner === owner);
  const newCubes = clickedCubes.map(cube => ({ center: [cube.position.x, cube.position.y, cube.position.z], side_length: cubeSize }));
  if (!regionColors.has(owner)) regionColors.set(owner, new THREE.Color(Math.random(), Math.random(), Math.random()));
  if (region) {
    const existingCenters = new Set(region.cubes.map(c => c.center.join(',')));
    const uniqueNewCubes = newCubes.filter(c => !existingCenters.has(c.center.join(',')));
    region.cubes.push(...uniqueNewCubes);
  } else {
    region = { owner, everyone_allowed: everyoneAllowed, authorized_players: [], cubes: newCubes };
    regions.push(region);
  }
  clickedCubes.forEach(cube => {
    cube.userData.clicked = false;
    cube.material.color.set(0x000000);
    cube.material.opacity = 0.05;
  });
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
    createCubesFromConfig(loadedRegions);
    updateRegionList();
  };
  reader.readAsText(file);
}

function createCubesFromConfig(loadedRegions) {
  loadedRegions.forEach(region => {
    const color = regionColors.get(region.owner) || new THREE.Color(Math.random(), Math.random(), Math.random());
    regionColors.set(region.owner, color);
    region.cubes.forEach(cubeData => {
      const geometry = new THREE.BoxGeometry(cubeData.side_length, cubeData.side_length, cubeData.side_length);
      const material = new THREE.MeshStandardMaterial({ color, transparent: true, opacity: 0.7 });
      const cube = new THREE.Mesh(geometry, material);
      cube.position.set(...cubeData.center);
      cube.userData.clicked = false;
      scene.add(cube);
      cubes.push(cube);
    });
  });
  updateWorldInfo();
}

function colorRegionCubes(region) {
  const color = regionColors.get(region.owner) || new THREE.Color(Math.random(), Math.random(), Math.random());
  regionColors.set(region.owner, color);
  region.cubes.forEach(cubeData => {
    const cube = cubes.find(c => c.position.x === cubeData.center[0] && c.position.y === cubeData.center[1] && c.position.z === cubeData.center[2]);
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
  const totalCubes = cubes.length;
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
  const intersects = raycaster.intersectObjects(cubes);
  if (intersects.length > 0) {
    const clickedObject = intersects[0].object;
    if (!clickedObject.userData.clicked) {
      clickedObject.material.color.set(0xff0000);
      clickedObject.material.opacity = 0.3; // Make the cube visible
      clickedObject.userData.clicked = true;
      setSelectedCube(clickedObject);
    } else {
      clickedObject.material.color.set(0x000000);
      clickedObject.material.opacity = 0; // Make the cube invisible again
      clickedObject.userData.clicked = false;
    }

    // Log the position of the clicked cube's center
    console.log(`Clicked cube at position: (${clickedObject.position.x}, ${clickedObject.position.y}, ${clickedObject.position.z})`);
  }
}

function onKeyDown(event) {
  if (!selectedCube) return;
  let { x, y, z } = selectedCube.position;
  const step = 16; // Each step is 16 units (one chunk)
  switch (event.key) {
    case ' ': selectedCube.userData.clicked = !selectedCube.userData.clicked; break;
    case 'a': x -= step; break;
    case 'd': x += step; break;
    case 'w': z -= step; break;
    case 's': z += step; break;
    case 'q': y -= step; break;
    case 'e': y += step; break;
    default: return;
  }
  const nextCube = cubes.find(cube => 
    Math.abs(cube.position.x - x) < 0.1 && 
    Math.abs(cube.position.y - y) < 0.1 && 
    Math.abs(cube.position.z - z) < 0.1
  );
  if (nextCube) setSelectedCube(nextCube);
}

function unselectCube() {
  if (selectedCube) {
    selectedCube.material.color.set(selectedCube.userData.clicked ? 0xff0000 : 0x000000);
    selectedCube.material.opacity = selectedCube.userData.clicked ? 0.3 : 0.05;
    selectedCube = null;
  }
}

function setSelectedCube(cube) {
  unselectCube();
  selectedCube = cube;
  selectedCube.material.color.set(0xfd7904);
  selectedCube.material.opacity = 0.3;

  // Log the position of the selected cube's center
  console.log(`Selected cube at position: (${selectedCube.position.x}, ${selectedCube.position.y}, ${selectedCube.position.z})`);
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
          createCubesBasedOnMinecraftWorld();
        },
        undefined,
        (error) => {
          console.error('Error loading OBJ file:', error);
          createCubes();
        }
      );
    },
    undefined,
    (error) => {
      console.error('Error loading MTL file:', error);
      createCubes();
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

function createCubesBasedOnMinecraftWorld() {
  const bbox = new THREE.Box3().setFromObject(minecraftWorld);
  const size = bbox.getSize(new THREE.Vector3());
  const center = bbox.getCenter(new THREE.Vector3());

  const cubeSize = 16; // Each cube represents a 16x16x16 block volume
  const gridX = Math.ceil(size.x / cubeSize);
  const gridY = Math.ceil(size.y / cubeSize);
  const gridZ = Math.ceil(size.z / cubeSize);

  const gridGroup = new THREE.Group(); // Create a group for the grid

  const offsetX = Math.floor(gridX / 2) * cubeSize;
  const offsetY = Math.floor(bbox.min.y / cubeSize) * cubeSize;
  const offsetZ = Math.floor(gridZ / 2) * cubeSize;

  for (let i = 0; i < gridX; i++) {
    for (let j = 0; j < gridY; j++) {
      for (let k = 0; k < gridZ; k++) {
        const geometry = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
        const material = new THREE.MeshStandardMaterial({ color: 0xffffff, transparent: true, opacity: 0 }); // Initially invisible
        const cube = new THREE.Mesh(geometry, material);
        
        // Position the cube center
        cube.position.set(
          i * cubeSize - offsetX + cubeSize / 2,
          j * cubeSize + offsetY + cubeSize / 2,
          k * cubeSize - offsetZ + cubeSize / 2
        );
        
        cube.userData.clicked = false;
        gridGroup.add(cube);
        cubes.push(cube);
      }
    }
  }

  // Center the Minecraft world
  minecraftWorld.position.set(-center.x, -bbox.min.y, -center.z);

  scene.add(gridGroup);
  scene.add(minecraftWorld);
  updateWorldInfo();
}

function fitCameraToObject(camera, object, offset = 1.25) {
  const boundingBox = new THREE.Box3().setFromObject(object);
  const center = boundingBox.getCenter(new THREE.Vector3());
  const size = boundingBox.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  const fov = camera.fov * (Math.PI / 180);
  let cameraZ = Math.abs(maxDim / 2 * Math.tan(fov * 2));
  cameraZ *= offset;
  camera.position.set(center.x, center.y, cameraZ);
  const minZ = boundingBox.min.z;
  const cameraToFarEdge = (minZ < 0) ? -minZ + cameraZ : cameraZ - minZ;
  camera.far = cameraToFarEdge * 3;
  camera.updateProjectionMatrix();
  if (controls) {
    controls.target.copy(center);
    controls.maxDistance = cameraToFarEdge * 2;
    controls.update();
  } else {
    camera.lookAt(center);
  }
}

function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}

