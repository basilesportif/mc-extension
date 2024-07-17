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
  const skyRef = useRef(null);

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
  const [isLocked, setIsLocked] = useState(false);
  const [showInstructions, setShowInstructions] = useState(true);
  const [shouldLoadWorld, setShouldLoadWorld] = useState(true);

  const [moveForward, setMoveForward] = useState(false);
  const [moveBackward, setMoveBackward] = useState(false);
  const [moveLeft, setMoveLeft] = useState(false);
  const [moveRight, setMoveRight] = useState(false);
  const [moveUp, setMoveUp] = useState(false);
  const [moveDown, setMoveDown] = useState(false);

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

  const onPointerLockChange = useCallback(() => {
    const isLocked = document.pointerLockElement === containerRef.current;
    setIsLocked(isLocked);
    setIsInteractive(isLocked);
    setIsMoving(isLocked);
    setShowMenu(!isLocked);
    if (!isLocked) {
      setMovement({
        forward: false,
        backward: false,
        left: false,
        right: false,
        up: false,
        down: false
      });
    }
  }, []);

  const onPointerLockError = useCallback(() => {
    console.error('PointerLock Error');
  }, []);

  const enterMovementMode = useCallback((event) => {
    if (controlsRef.current && document.pointerLockElement !== containerRef.current) {
      containerRef.current.requestPointerLock();
      setShowInstructions(false);
      setIsLocked(true);
    }
  }, []);

  const exitMovementMode = useCallback(() => {
    if (controlsRef.current) {
      controlsRef.current.unlock();
    }
    setShowInstructions(true);
    setIsInteractive(false);
    setIsMoving(false);
    setMovement({
      forward: false,
      backward: false,
      left: false,
      right: false,
      up: false,
      down: false
    });
    setShouldLoadWorld(false);
    setIsLocked(false);
  }, []);

  const handleKeyDown = useCallback((event) => {
    if (!isLocked) return;
    switch (event.code) {
      case 'ArrowUp':
      case 'KeyW': setMoveForward(true); break;
      case 'ArrowLeft':
      case 'KeyA': setMoveLeft(true); break;
      case 'ArrowDown':
      case 'KeyS': setMoveBackward(true); break;
      case 'ArrowRight':
      case 'KeyD': setMoveRight(true); break;
      case 'Space': setMoveUp(true); break;
      case 'ShiftLeft': setMoveDown(true); break;
      case 'Escape':
        exitMovementMode();
        event.preventDefault();
        break;
      default:
        break;
    }
  }, [isLocked, exitMovementMode]);

  const handleKeyUp = useCallback((event) => {
    if (!isLocked) return;
    switch (event.code) {
      case 'ArrowUp':
      case 'KeyW': setMoveForward(false); break;
      case 'ArrowLeft':
      case 'KeyA': setMoveLeft(false); break;
      case 'ArrowDown':
      case 'KeyS': setMoveBackward(false); break;
      case 'ArrowRight':
      case 'KeyD': setMoveRight(false); break;
      case 'Space': setMoveUp(false); break;
      case 'ShiftLeft': setMoveDown(false); break;
      default:
        break;
    }
  }, [isLocked]);

  useEffect(() => {
    const handleKeyDownListener = (event) => handleKeyDown(event);
    const handleKeyUpListener = (event) => handleKeyUp(event);

    if (isLocked) {
      window.addEventListener('keydown', handleKeyDownListener);
      window.addEventListener('keyup', handleKeyUpListener);
    }

    return () => {
      window.removeEventListener('keydown', handleKeyDownListener);
      window.removeEventListener('keyup', handleKeyUpListener);
    };
  }, [isLocked, handleKeyDown, handleKeyUp]);

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

  const handleMovement = useCallback(() => {
    if (!isLocked || !controlsRef.current) return;

    const currentTime = performance.now();
    const delta = (currentTime - prevTime) / 1000;

    velocity.x -= velocity.x * 10.0 * delta;
    velocity.z -= velocity.z * 10.0 * delta;
    velocity.y -= velocity.y * 10.0 * delta;

    direction.z = Number(moveForward) - Number(moveBackward);
    direction.x = Number(moveRight) - Number(moveLeft);
    direction.y = Number(moveUp) - Number(moveDown);
    direction.normalize();

    const speed = 400.0;
    if (moveForward || moveBackward) velocity.z -= direction.z * speed * delta;
    if (moveLeft || moveRight) velocity.x -= direction.x * speed * delta;
    if (moveUp || moveDown) velocity.y += direction.y * speed * delta;
    // remember to try without the if statement
    if (controlsRef.current) {
      controlsRef.current.moveRight(-velocity.x * delta);
      controlsRef.current.moveForward(-velocity.z * delta);
      controlsRef.current.getObject().position.y += velocity.y * delta;
    }

    // Keep the sky centered on the camera
    if (skyRef.current) {
      skyRef.current.position.copy(controlsRef.current.getObject().position);
    }

    setPrevTime(currentTime);
  }, [isLocked, moveForward, moveBackward, moveLeft, moveRight, moveUp, moveDown, prevTime]);

  const handleAnimationFrame = useCallback(() => {
    handleMovement();
    if (rendererRef.current && sceneRef.current && cameraRef.current) {
      rendererRef.current.render(sceneRef.current, cameraRef.current);
    }
    animationFrameRef.current = requestAnimationFrame(handleAnimationFrame);
  }, [handleMovement]);

  useEffect(() => {
    if (initialized) {
      handleAnimationFrame();
    }
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [initialized, handleAnimationFrame]);

  const init = async () => {
    if (sceneRef.current) {
      console.log('Scene already initialized.');
      return;
    }

    try {
      console.log('Initializing Three.js scene...');
      setupScene();
      setupCamera();
      setupRenderer();
      setupSky();
      setupLighting();
      createAxes();
      
      console.log('Loading Minecraft world...');
      await loadMinecraftWorld();
      
      console.log('Three.js scene initialized.');
      setInitialized(true);
      setupPointerLockControls();
    } catch (error) {
      console.error('Error initializing scene:', error);
      // Handle the error appropriately (e.g., show user message)
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
    skyRef.current = sky;

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
    if (cameraRef.current && containerRef.current) {
      const controls = new PointerLockControls(cameraRef.current, containerRef.current);
      controlsRef.current = controls;

      controls.addEventListener('lock', () => {
        setShowInstructions(false);
        setIsLocked(true);
      });

      controls.addEventListener('unlock', () => {
        setShowInstructions(true);
        setIsLocked(false);
        setMoveForward(false);
        setMoveBackward(false);
        setMoveLeft(false);
        setMoveRight(false);
        setMoveUp(false);
        setMoveDown(false);
      });

      sceneRef.current.add(controls.getObject());
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
      if (!shouldLoadWorld) {
        // If the user has exited the interactive mode, don't load the world
        resolve();
        return;
      }

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

  useEffect(() => {
    if (!initialized) {
      init();
    }
  }, [initialized]);

  useEffect(() => {
    const handlePointerLockChange = () => {
      const isLocked = document.pointerLockElement === containerRef.current;
      setIsLocked(isLocked);
      setIsInteractive(isLocked);
      setIsMoving(isLocked);
      if (!isLocked) {
        setShowInstructions(true);
        setMovement({
          forward: false,
          backward: false,
          left: false,
          right: false,
          up: false,
          down: false
        });
      }
    };

    document.addEventListener('pointerlockchange', handlePointerLockChange);

    return () => {
      document.removeEventListener('pointerlockchange', handlePointerLockChange);
    };
  }, []);

  useEffect(() => {
    if (initialized) {
      document.addEventListener('pointerlockchange', onPointerLockChange);
      document.addEventListener('pointerlockerror', onPointerLockError);

      if (controlsRef.current) {
        sceneRef.current.add(controlsRef.current.getObject());
      }

      handleAnimationFrame();
    }

    return () => {
      document.removeEventListener('pointerlockchange', onPointerLockChange);
      document.removeEventListener('pointerlockerror', onPointerLockError);
      cancelAnimationFrame(animationFrameRef.current);
    };
  }, [initialized, onPointerLockChange, onPointerLockError]);

  useEffect(() => {
    if (initialized && containerRef.current) {
      const handleUserInteraction = (event) => {
        if (event.target === containerRef.current && !isLocked) {
          enterMovementMode(event);
        }
      };

      containerRef.current.addEventListener('click', handleUserInteraction);

      return () => {
        if (containerRef.current) {
          containerRef.current.removeEventListener('click', handleUserInteraction);
        }
      };
    }
  }, [initialized, isLocked, enterMovementMode]);

  return (
    <div 
      ref={containerRef} 
      style={{ width: '100%', height: '100%', position: 'relative' }}
      onClick={enterMovementMode}
      tabIndex="0"
    >
      {showInstructions && (
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
        >
          <div>Click to Start</div>
          <div style={{ fontSize: '18px', marginTop: '20px' }}>
            WASD or Arrow keys to move, Space to go up, Shift to go down
            <br />
            Mouse to look around, ESC to exit, Click to resume
          </div>
        </div>
      )}
    </div>
  );
};

export default ThreeJsScene;
