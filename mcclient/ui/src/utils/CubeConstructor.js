
import * as THREE from "three";
export const CubeConstructor = (
    minecraftWorldRef,
    sceneRef,
    cubesRef
  ) => {
    if (!minecraftWorldRef.current || !sceneRef.current) return;
  
    const minecraftWorld = minecraftWorldRef.current;
    const scene = sceneRef.current;
  
    const bbox = new THREE.Box3().setFromObject(minecraftWorld);
    const size = bbox.getSize(new THREE.Vector3());
    const center = bbox.getCenter(new THREE.Vector3());
  
    const cubeSize = 16;
    const gridX = Math.ceil(size.x / cubeSize);
    const gridY = Math.ceil(size.y / cubeSize);
    const gridZ = Math.ceil(size.z / cubeSize);
  
    const gridGroup = new THREE.Group();
  
    const offsetX = Math.floor(gridX / 2) * cubeSize;
    const offsetY = Math.floor(bbox.min.y / cubeSize) * cubeSize;
    const offsetZ = Math.floor(gridZ / 2) * cubeSize;
  
    const newCubes = [];
  
    for (let i = 0; i < gridX; i++) {
      for (let j = 0; j < gridY; j++) {
        for (let k = 0; k < gridZ; k++) {
          const geometry = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
          const material = new THREE.MeshPhongMaterial({
            color: 0xffffff,
            transparent: true,
            opacity: 0.02,
            side: THREE.DoubleSide,
          });
          const cube = new THREE.Mesh(geometry, material);
  
          const edgesGeometry = new THREE.EdgesGeometry(geometry);
          const edgesMaterial = new THREE.LineBasicMaterial({
            color: 0xcccccc,
            transparent: true,
            opacity: 0.1,
            linewidth: 1,
          });
          const wireframe = new THREE.LineSegments(edgesGeometry, edgesMaterial);
          cube.add(wireframe);
  
          cube.position.set(
            i * cubeSize - offsetX + cubeSize / 2,
            j * cubeSize + offsetY + cubeSize / 2,
            k * cubeSize - offsetZ + cubeSize / 2
          );
  
          cube.userData.clicked = false;
          gridGroup.add(cube);
          newCubes.push(cube);
        }
      }
    }
  
    minecraftWorld.position.set(-center.x, -bbox.min.y, -center.z);
  
    scene.add(gridGroup);
    cubesRef.current = newCubes;
    console.log("Cubes created based on Minecraft world.");
  };