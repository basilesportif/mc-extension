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
let cubeSize = 50; // Initial cube size
const cubes = [];
const regions = []; // List to store regions
const regionColors = new Map(); // Map to store colors for each region

document.addEventListener('DOMContentLoaded', () => {
  init();
  animate();
});

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

  createCubes();
  createAxes();

  const axesHelper = new THREE.AxesHelper(200);
  axesHelper.setColors(0xffffff, 0xffffff, 0xffffff);
  scene.add(axesHelper);

  // Event listeners
  window.addEventListener('resize', onWindowResize, false);
  window.addEventListener('mousemove', onMouseMove, false);
  window.addEventListener('click', onMouseClick, false);
  window.addEventListener('keydown', onKeyDown, false);

  // Add button event listeners
  document.getElementById('clearSelectionButton').addEventListener('click', clearSelection);
  document.getElementById('addRegionButton').addEventListener('click', addRegion);
  document.getElementById('generateWorldConfigButton').addEventListener('click', generateWorldConfig);
  document.getElementById('cubeSizeInput').addEventListener('change', onCubeSizeChange);
  document.getElementById('loadConfigButton').addEventListener('change', loadConfig);

  // Update world info
  updateWorldInfo();
}

function createCubes() {
  // Remove existing cubes
  cubes.forEach(cube => scene.remove(cube));
  cubes.length = 0;

  // Create new cubes
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
        cube.position.set(
          i * cubeSize - (gridSize * cubeSize) / 2 + cubeSize / 2,
          k * cubeSize + cubeSize / 2,
          j * cubeSize - (gridSize * cubeSize) / 2 + cubeSize / 2
        );
        cube.userData.clicked = false;  // Track if the cube has been clicked
        scene.add(cube);
        cubes.push(cube);
      }
    }
  }

  // Update world info
  updateWorldInfo();
}

function createAxes() {
  const material = new THREE.LineBasicMaterial({ color: 0xff0000 });
  const largeNumber = 10000; // Use a large number instead of Infinity

  const pointsX = [];
  pointsX.push(new THREE.Vector3(-largeNumber, 0, 0));
  pointsX.push(new THREE.Vector3(largeNumber, 0, 0));

  const pointsY = [];
  pointsY.push(new THREE.Vector3(0, -largeNumber, 0));
  pointsY.push(new THREE.Vector3(0, largeNumber, 0));

  const pointsZ = [];
  pointsZ.push(new THREE.Vector3(0, 0, -largeNumber));
  pointsZ.push(new THREE.Vector3(0, 0, largeNumber));

  const geometryX = new THREE.BufferGeometry().setFromPoints(pointsX);
  const geometryY = new THREE.BufferGeometry().setFromPoints(pointsY);
  const geometryZ = new THREE.BufferGeometry().setFromPoints(pointsZ);

  const lineX = new THREE.Line(geometryX, material);
  const lineY = new THREE.Line(geometryY, material);
  const lineZ = new THREE.Line(geometryZ, material);

  scene.add(lineX);
  scene.add(lineY);
  scene.add(lineZ);
}

function onCubeSizeChange(event) {
  cubeSize = parseInt(event.target.value);
  createCubes();
}

function clearSelection() {
  // Clear the regions array
  regions.length = 0;

  // Reset the cubes
  cubes.forEach(cube => {
    cube.userData.clicked = false;
    cube.material.color.set(0x000000);
    cube.material.opacity = 0.1;
  });
  selectedCube = null;
  console.log('Selection cleared');

  // Clear the region list
  const regionList = document.getElementById('regionList');
  regionList.innerHTML = '';

  // Clear the loaded file input
  const loadConfigButton = document.getElementById('loadConfigButton');
  loadConfigButton.value = '';

  // Recreate the initial cubes
  createCubes();
}

function addRegion() {
  const clickedCubes = cubes.filter(cube => cube.userData.clicked);
  if (clickedCubes.length === 0) {
    alert('No cubes selected!');
    return;
  }

  const owner = prompt('Enter region owner:');
  if (!owner) {
    alert('Owner name is required!');
    return;
  }

  const everyoneAllowed = confirm('Do you authorize all players to roam freely in your region? Click "OK" for Yes and "Cancel" for No.');
  const vectorString = prompt('Enter a vector string:');

  let region = regions.find(r => r.owner === owner);
  const newCubes = clickedCubes.map(cube => ({
    center: [cube.position.x, cube.position.y, cube.position.z],
    side_length: cubeSize
  }));

  if (!regionColors.has(owner)) {
    regionColors.set(owner, new THREE.Color(Math.random(), Math.random(), Math.random()));
  }

  if (region) {
    // Only add cubes that are not already in the region
    const existingCenters = new Set(region.cubes.map(c => c.center.join(',')));
    const uniqueNewCubes = newCubes.filter(c => !existingCenters.has(c.center.join(',')));
    region.cubes.push(...uniqueNewCubes);
  } else {
    region = {
      owner: owner,
      everyone_allowed: everyoneAllowed,
      authorized_players: [],
      cubes: newCubes
    };
    regions.push(region);
  }

  // Mark the cubes as part of a region
  clickedCubes.forEach(cube => {
    cube.userData.clicked = false;
    cube.material.color.set(0x000000);
    cube.material.opacity = 0.1;
  });

  updateRegionList();
  colorRegionCubes(region);
  console.log('Region added:', JSON.stringify(region, null, 2));
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
  console.log('World config generated and downloaded');
}

function loadConfig(event) {
  const file = event.target.files[0];
  if (!file) {
    return;
  }

  const reader = new FileReader();
  reader.onload = function(e) {
    const content = e.target.result;
    const loadedRegions = JSON.parse(content);
    regions.length = 0; // Clear existing regions
    regions.push(...loadedRegions);
    createCubesFromConfig(loadedRegions);
    updateRegionList();
  };
  reader.readAsText(file);
}

function createCubesFromConfig(loadedRegions) {
  loadedRegions.forEach(region => {
    let color;
    if (regionColors.has(region.owner)) {
      color = regionColors.get(region.owner);
    } else {
      color = new THREE.Color(Math.random(), Math.random(), Math.random());
      regionColors.set(region.owner, color);
    }

    region.cubes.forEach(cubeData => {
      const geometry = new THREE.BoxGeometry(cubeData.side_length, cubeData.side_length, cubeData.side_length);
      const material = new THREE.MeshStandardMaterial({
        color: color,
        transparent: true,
        opacity: 0.7
      });
      const cube = new THREE.Mesh(geometry, material);
      cube.position.set(...cubeData.center);
      cube.userData.clicked = false;  // Track if the cube has been clicked
      scene.add(cube);
      cubes.push(cube);
    });
  });

  // Update world info
  updateWorldInfo();
}

function colorRegionCubes(region) {
  let color;
  if (regionColors.has(region.owner)) {
    color = regionColors.get(region.owner);
  } else {
    color = new THREE.Color(Math.random(), Math.random(), Math.random());
    regionColors.set(region.owner, color);
  }

  region.cubes.forEach(cubeData => {
    const cube = cubes.find(c => c.position.x === cubeData.center[0] && c.position.y === cubeData.center[1] && c.position.z === cubeData.center[2]);
    if (cube) {
      cube.material.color.set(color);
      cube.material.opacity = 0.7;
    }
  });

  // Update world info
  updateWorldInfo();
}
function updateRegionList() {
  const regionList = document.getElementById('regionList');
  regionList.innerHTML = '';

  const ownerCubeCount = new Map();

  regions.forEach(region => {
    const cubeCount = region.cubes.length;
    ownerCubeCount.set(region.owner, (ownerCubeCount.get(region.owner) || 0) + cubeCount);
    
    // Ensure a color is assigned to the owner if it doesn't exist
    if (!regionColors.has(region.owner)) {
      regionColors.set(region.owner, new THREE.Color(Math.random(), Math.random(), Math.random()));
    }
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

  // Update world info
  updateWorldInfo();
}

function updateWorldInfo() {
  const totalCubes = gridSize * gridSize * height;
  const usedCubes = regions.reduce((sum, region) => sum + region.cubes.length, 0);
  const availableCubes = totalCubes - usedCubes;
  const worldSize = `${gridSize * cubeSize} x ${height * cubeSize} x ${gridSize * cubeSize}`;

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
    let x = selectedCube.position.x;
    let y = selectedCube.position.y;
    let z = selectedCube.position.z;
    console.log(`x: ${x}, y: ${y}, z: ${z}`)

    const step = cubeSize; // Adjust step to match the new cube size

    switch (event.key) {
      case ' ':
        selectedCube.userData.clicked = !selectedCube.userData.clicked;
        break;
      case 'a':
        x -= step;
        break;
      case 'd':
        x += step;
        break;
      case 'w':
        z -= step;
        break;
      case 's':
        z += step;
        break;
      case 'q':
        y -= step;
        break;
      case 'e':
        y += step;
        break;
      default:
        return;
    }
    const nextCube = cubes.find(cube => cube.position.x === x && cube.position.y === y && cube.position.z === z);
    if (nextCube) {
      setSelectedCube(nextCube);
    }
  }
}

function unselectCube() {
  if (selectedCube) {
    if (selectedCube.userData.clicked) {
      selectedCube.material.color.set(0xff0000);
      selectedCube.material.opacity = 0.7; // Set to red if clicked
    }
    else {
      selectedCube.material.color.set(0x000000);
      selectedCube.material.opacity = 0.1;
    }
    selectedCube = null;
  }
}

function setSelectedCube(cube) {
  unselectCube();
  selectedCube = cube;
  selectedCube.material.color.set(0xfd7904);
  selectedCube.material.opacity = 0.7;// Set to orange if selected
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
