import React, { useEffect, useRef, useState, useCallback } from 'react';
import * as THREE from 'three';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js';
import { PointerLockControls } from 'three/examples/jsm/controls/PointerLockControls.js';
import { Sky } from 'three/examples/jsm/objects/Sky.js';

const ThreeJsScene = () => {
  const mountRef = useRef(null);
  const sceneRef = useRef(null);
  const cameraRef = useRef(null);
  const rendererRef = useRef(null);
  const controlsRef = useRef(null);
  const minecraftWorldRef = useRef(null);
  const cubesRef = useRef([]);
  const animationFrameRef = useRef(null);
  const containerRef = useRef(null);

  const [isInteractive, setIsInteractive] = useState(false);
  const [isMoving, setIsMoving] = useState(false);
  const [showMenu, setShowMenu] = useState(true);
  const [movement, setMovement] = useState({
    forward: false,
    backward: false,
    left: false,
    right: false,
    up: false,
    down: false
  });
  const [selectedCube, setSelectedCube] = useState(null);
  const [velocity] = useState(new THREE.Vector3());
  const [direction] = useState(new THREE.Vector3());
  const [prevTime, setPrevTime] = useState(performance.now());
  const [initialized, setInitialized] = useState(false);


  const setupRenderer = useCallback(() => {
    if (!rendererRef.current && containerRef.current) {
      const renderer = new THREE.WebGLRenderer({ antialias: true });
      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;
      renderer.setSize(width, height);
      renderer.setPixelRatio(window.devicePixelRatio);
      rendererRef.current = renderer;
      containerRef.current.appendChild(renderer.domElement);
      console.log('Renderer set up.');
    }
  }, []);

  const setupCamera = useCallback(() => {
    if (containerRef.current) {
      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;
      const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
      camera.position.set(0, 50, 200);
      cameraRef.current = camera;
      console.log('Camera set up.');
    }
  }, []);

  const enterMovementMode = async () => {
    if (!initialized) {
      console.log('Initializing Three.js scene...');
      setupRenderer(); // Set up the renderer first
      await init(); // Ensure initialization is complete before entering movement mode
    }
    setIsInteractive(true);
    setIsMoving(true);
    setShowMenu(false);
    if (controlsRef.current) {
      controlsRef.current.lock();
    }
  };

  const exitMovementMode = useCallback(() => {
    setIsMoving(false);
    setShowMenu(true);
    if (controlsRef.current) {
      controlsRef.current.unlock();
    }
  }, []);

  const handleKeyDown = useCallback((event) => {
    switch (event.code) {
      case 'KeyW':
        setMovement(prev => ({ ...prev, forward: true }));
        break;
      case 'KeyS':
        setMovement(prev => ({ ...prev, backward: true }));
        break;
      case 'KeyA':
        setMovement(prev => ({ ...prev, left: true }));
        break;
      case 'KeyD':
        setMovement(prev => ({ ...prev, right: true }));
        break;
      case 'Space':
        setMovement(prev => ({ ...prev, up: true }));
        break;
      case 'ShiftLeft':
        setMovement(prev => ({ ...prev, down: true }));
        break;
      case 'Escape':
        exitMovementMode();
        break;
      default:
        break;
    }
  }, [exitMovementMode]);

  const handleKeyUp = useCallback((event) => {
    switch (event.code) {
      case 'KeyW':
        setMovement(prev => ({ ...prev, forward: false }));
        break;
      case 'KeyS':
        setMovement(prev => ({ ...prev, backward: false }));
        break;
      case 'KeyA':
        setMovement(prev => ({ ...prev, left: false }));
        break;
      case 'KeyD':
        setMovement(prev => ({ ...prev, right: false }));
        break;
      case 'Space':
        setMovement(prev => ({ ...prev, up: false }));
        break;
      case 'ShiftLeft':
        setMovement(prev => ({ ...prev, down: false }));
        break;
      default:
        break;
    }
  }, []);

  useEffect(() => {
    const handleKeyUpListener = (event) => handleKeyUp(event);
    window.addEventListener('keyup', handleKeyUpListener);

    return () => {
      window.removeEventListener('keyup', handleKeyUpListener);
    };
  }, [handleKeyUp]);

  const handleMouseClick = useCallback((event) => {
    const { clientX, clientY } = event;
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();
    mouse.x = (clientX / window.innerWidth) * 2 - 1;
    mouse.y = -(clientY / window.innerHeight) * 2 + 1;

    if (cameraRef.current && sceneRef.current) {
      raycaster.setFromCamera(mouse, cameraRef.current);

      const intersects = raycaster.intersectObjects(sceneRef.current.children, true);
      if (intersects.length > 0) {
        const intersectedObject = intersects[0].object;
        if (intersectedObject.userData.isMinecraftWorld) {
          if (selectedCube) {
            selectedCube.material.opacity = 1;
          }
          setSelectedCube(intersectedObject);
          intersectedObject.material.opacity = 0.5;
        }
      }
    }
  }, [isMoving, selectedCube]);

  useEffect(() => {
    if (selectedCube) {
      selectedCube.material.opacity = 0.5;
    }
  }, [selectedCube]);

 

  const handleMovement = () => {
    if (!isMoving) return;

    const currentTime = performance.now();
    const delta = (currentTime - prevTime) / 1000;

    velocity.x -= velocity.x * 10.0 * delta;
    velocity.z -= velocity.z * 10.0 * delta;
    velocity.y -= velocity.y * 10.0 * delta;

    direction.z = Number(movement.forward) - Number(movement.backward);
    direction.x = Number(movement.right) - Number(movement.left);
    direction.y = Number(movement.up) - Number(movement.down);
    direction.normalize();

    const speed = 5.0;
    if (movement.forward || movement.backward) velocity.z -= direction.z * speed * delta;
    if (movement.left || movement.right) velocity.x -= direction.x * speed * delta;
    if (movement.up || movement.down) velocity.y += direction.y * speed * delta;

    if (controlsRef.current) {
      controlsRef.current.moveRight(-velocity.x * delta);
      controlsRef.current.moveForward(-velocity.z * delta);
      const newPosition = controlsRef.current.getObject().position.clone();
      newPosition.y += velocity.y * delta;
      controlsRef.current.getObject().position.copy(newPosition);
    }

    handlePlayerPosition(delta);

    setPrevTime(currentTime);
  };

  const handlePlayerPosition = (delta) => {
    if (controlsRef.current && controlsRef.current.getObject()) {
      if (controlsRef.current.getObject().position.y < 0) {
        velocity.y = 0;
        const newPosition = controlsRef.current.getObject().position.clone();
        newPosition.y = 0;
        controlsRef.current.getObject().position.copy(newPosition);
      }
      velocity.y -= 9.8 * delta;
    } else {
      velocity.y -= 9.8 * delta;
    }
  };

  const handleAnimationFrame = () => {
    animationFrameRef.current = requestAnimationFrame(handleAnimationFrame);
    handleMovement();
    if (rendererRef.current && sceneRef.current && cameraRef.current) {
      rendererRef.current.render(sceneRef.current, cameraRef.current);
    }
  };

  const init = async () => {
    if (!sceneRef.current) {
      setupScene();
      setupCamera();
      setupSky();
      setupLighting();
      createAxes();
      console.log('Loading Minecraft world...');
      await loadMinecraftWorld();
      setupPointerLockControls();
      console.log('Three.js scene initialized.');
      setInitialized(true);
    }
  };

  const setupScene = () => {
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x87CEEB);
    scene.fog = new THREE.FogExp2(0x87CEEB, 0.00025);
    sceneRef.current = scene;
    console.log('Scene set up.');
  };

  const setupSky = () => {
    const sky = new Sky();
    sky.scale.setScalar(450000);
    sceneRef.current.add(sky);

    const sun = new THREE.Vector3();
    const effectController = {
      turbidity: 10,
      rayleigh: 2,
      mieCoefficient: 0.005,
      mieDirectionalG: 0.8,
      elevation: 2,
      azimuth: 180,
      exposure: rendererRef.current.toneMappingExposure
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
    console.log('Sky set up.');
  };

  const setupPointerLockControls = () => {
    if (cameraRef.current && rendererRef.current) {
      const controls = new PointerLockControls(cameraRef.current, rendererRef.current.domElement);
      controlsRef.current = controls;
      console.log('PointerLockControls set up.');
    }
  };

  const setupLighting = () => {
    const ambientLight = new THREE.AmbientLight(0x404040);
    sceneRef.current.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5);
    directionalLight.position.set(1, 1, 1);
    sceneRef.current.add(directionalLight);
    console.log('Lighting set up.');
  };

  const createAxes = () => {
    const material = new THREE.LineBasicMaterial({ color: 0xff0000 });
    const largeNumber = 10000;
    const axes = ['x', 'y', 'z'].map(axis => {
      const points = [new THREE.Vector3(), new THREE.Vector3()];
      points[0][axis] = -largeNumber;
      points[1][axis] = largeNumber;
      const geometry = new THREE.BufferGeometry().setFromPoints(points);
      return new THREE.Line(geometry, material);
    });
    sceneRef.current.add(...axes);
    console.log('Axes created.');
  };

  const loadMinecraftWorld = () => {
    return new Promise((resolve, reject) => {
      console.log('Loading MTL file...');
      const objLoader = new OBJLoader();
      const mtlLoader = new MTLLoader();
      const objPath = '/mcclient:mcclient:basilesex.os/minecraft.obj';
      const mtlPath = '/mcclient:mcclient:basilesex.os/minecraft.mtl';

      mtlLoader.load(
        mtlPath,
        (materials) => {
          console.log('MTL file loaded.');
          materials.preload();
          objLoader.setMaterials(materials);
          console.log('Loading OBJ file...');
          objLoader.load(
            objPath,
            (object) => {
              console.log('OBJ file loaded.');
              object.traverse((child) => {
                if (child.isMesh) {
                  child.userData.isMinecraftWorld = true;
                  if (child.material.map) {
                    const textureType = getTextureType(child.material.map.name);
                    console.log(`Texture type: ${textureType}`);
                  }
                }
              });
              sceneRef.current.add(object);
              minecraftWorldRef.current = object;
              console.log('Minecraft world added to scene.');
              resolve();
            },
            undefined,
            (error) => {
              console.error('Error loading OBJ file:', error);
              reject(error);
            }
          );
        },
        undefined,
        (error) => {
          console.error('Error loading MTL file:', error);
          reject(error);
        }
      );
    });
  };

  const getTextureType = (textureName) => {
    if (textureName.includes('block')) return 'block';
    if (textureName.includes('entity')) return 'entity';
    if (textureName.includes('painting')) return 'painting';
    if (textureName.includes('banner')) return 'banner';
    if (textureName.includes('models')) return 'models';
    return 'block';
  };

  return (
    <div 
      ref={containerRef} 
      style={{ width: '100%', height: '100%', position: 'relative' }}
      onClick={async () => {
        if (isInteractive && !isMoving) {
          await enterMovementMode();
        } else if (showMenu) {
          setShowMenu(false);
          await enterMovementMode();
        }
      }}
    >
      {showMenu && (
        <div 
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'center',
            alignItems: 'center',
            background: 'rgba(0,0,0,0.5)',
            color: 'white',
            fontSize: '24px',
            cursor: 'pointer',
          }}
          onClick={enterMovementMode}
        >
          <div>{isInteractive ? 'Click to Resume' : 'Click to Start'}</div>
          <div style={{ fontSize: '18px', marginTop: '20px' }}>
            WASD to move, Space to go up, Shift to go down
            <br />
            Mouse to look around, ESC to stop, Click to resume
          </div>
        </div>
      )}
    </div>
  );
};

export default ThreeJsScene;