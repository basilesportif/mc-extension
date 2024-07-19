import React, { useEffect, useRef, useState, useCallback } from "react";
import * as THREE from "three";
import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
import { MTLLoader } from "three/examples/jsm/loaders/MTLLoader.js";
import { PointerLockControls } from "three/examples/jsm/controls/PointerLockControls.js";
import { Sky } from "three/examples/jsm/objects/Sky.js";

const ThreeJsScene = ({ ws, ourInTeam, lobby }) => {
  const sceneRef = useRef(null);
  const cameraRef = useRef(null);
  const rendererRef = useRef(null);
  const controlsRef = useRef(null);
  const minecraftWorldRef = useRef(null);
  const cubesRef = useRef([]);
  const animationFrameRef = useRef(null);
  const containerRef = useRef(null);
  const skyRef = useRef(null);

  const [movement, setMovement] = useState({
    forward: false,
    backward: false,
    left: false,
    right: false,
    up: false,
    down: false,
  });

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

  const velocityRef = useRef(new THREE.Vector3());
  const directionRef = useRef(new THREE.Vector3());
  const prevTimeRef = useRef(performance.now());

  const [selectedCubes, setSelectedCubes] = useState([]);



  const [showTeamAlert, setShowTeamAlert] = useState(false);
  const [showCrosshair, setShowCrosshair] = useState(false);

  const init = async () => {
    if (sceneRef.current) {
      console.log("Scene already initialized.");
      return;
    }

    try {
      console.log("Initializing Three.js scene...");
      setupScene();
      setupCamera();
      setupRenderer();
      setupSky();
      setupLighting();
      createAxes();

      console.log("Loading Minecraft world...");
      await loadMinecraftWorld();

      console.log("Three.js scene initialized.");
      setInitialized(true);
      setupPointerLockControls();
    } catch (error) {
      console.error("Error initializing scene:", error);
    }
  };
  const setupScene = () => {
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x87ceeb);
    scene.fog = new THREE.FogExp2(0x87ceeb, 0.00025);
    sceneRef.current = scene;
    console.log("Scene set up.");
  };

  const setupCamera = useCallback(() => {
    if (containerRef.current) {
      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;
      const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
      camera.position.set(0, 50, 200);
      cameraRef.current = camera;
      console.log("Camera set up.");
    }
  }, []);

  const setupRenderer = useCallback(() => {
    if (!rendererRef.current && containerRef.current) {
      const renderer = new THREE.WebGLRenderer({ antialias: true });
      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;
      renderer.setSize(width, height);
      renderer.setPixelRatio(window.devicePixelRatio);
      rendererRef.current = renderer;
      containerRef.current.appendChild(renderer.domElement);
      console.log("Renderer set up.");
    }
  }, []);

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

  const setupLighting = () => {
    // Ambient light
    const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
    sceneRef.current.add(ambientLight);

    // Directional light (sun-like)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(100, 100, 50);
    directionalLight.castShadow = true;
    sceneRef.current.add(directionalLight);

    console.log("Lighting set up.");
  };


  const createAxes = () => {
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

  const loadMinecraftWorld = () => {
    return new Promise((resolve, reject) => {
      if (!shouldLoadWorld) {
        resolve();
        return;
      }

      console.log("Loading MTL file...");
      const objLoader = new OBJLoader();
      const mtlLoader = new MTLLoader();
      const objPath = "/mcclient:mcclient:basilesex.os/minecraft.obj";
      const mtlPath = "/mcclient:mcclient:basilesex.os/minecraft.mtl";

      mtlLoader.load(
        mtlPath,
        (materials) => {
          console.log("MTL file loaded.");
          materials.preload();
          objLoader.setMaterials(materials);
          console.log("Loading OBJ file...");
          objLoader.load(
            objPath,
            (object) => {
              console.log("OBJ file loaded.");
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
              console.log("Minecraft world added to scene.");
              createCubesBasedOnMinecraftWorld();
              resolve();
            },
            undefined,
            (error) => {
              console.error("Error loading OBJ file:", error);
              reject(error);
            }
          );
        },
        undefined,
        (error) => {
          console.error("Error loading MTL file:", error);
          reject(error);
        }
      );
    });
  };

  const setupPointerLockControls = () => {
    if (cameraRef.current && containerRef.current) {
      const controls = new PointerLockControls(
        cameraRef.current,
        containerRef.current
      );
      controlsRef.current = controls;

      controls.addEventListener("lock", () => {
        setShowInstructions(false);
        setIsLocked(true);
      });

      controls.addEventListener("unlock", () => {
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
      console.log("PointerLockControls set up.");
    }
  };

  const onPointerLockChange = useCallback(() => {
    const isLocked = document.pointerLockElement === containerRef.current;
    setIsLocked(isLocked);
    
    if (!isLocked) {
      setMovement({
        forward: false,
        backward: false,
        left: false,
        right: false,
        up: false,
        down: false,
      });
    }
  }, []);

  const onPointerLockError = useCallback(() => {
    console.error("PointerLock Error");
  }, []);

  const enterMovementMode = useCallback(
    (event) => {
      if (!ourInTeam) {
        setShowTeamAlert(true);
        setTimeout(() => setShowTeamAlert(false), 3000); // Hide alert after 3 seconds
        return;
      }

      if (
        controlsRef.current &&
        document.pointerLockElement !== containerRef.current
      ) {
        containerRef.current.requestPointerLock();
        setShowInstructions(false);
        setIsLocked(true);
        
      }
    },
    [ourInTeam]
  );

  const exitMovementMode = useCallback(() => {
    if (controlsRef.current) {
      document.exitPointerLock();
    }
    setShowInstructions(true);
    setMovement({
      forward: false,
      backward: false,
      left: false,
      right: false,
      up: false,
      down: false,
    });
    setShouldLoadWorld(false);
    setIsLocked(false);
  }, []);

  const handleKeyDown = useCallback(
    (event) => {
      if (!isLocked) return;
      switch (event.code) {
        case "Enter":
          if (selectedCubes.length > 0) {
            // the exact format which backend needs
            const teamCubes = selectedCubes.reduce((acc, cube) => {
              const center = [
                cube.position.x,
                cube.position.y,
                cube.position.z,
              ];
              const side_length = 16; // Assuming a fixed side length
              const element = [{ center, side_length }, [["Slowness"]]];
              // if cube in list, dont add
              if (
                !acc.some(
                  (e) =>
                    e[0][0] === center[0] &&
                    e[0][1] === center[1] &&
                    e[0][2] === center[2] &&
                    e[0][3] === side_length
                )
              ) {
                acc.push(element);
              }
              return acc;
            }, []);

            let team =
              ourInTeam === "team1"
                ? "Team1"
                : ourInTeam === "team2"
                ? "Team2"
                : null;

            // this is the exact format the backend needs
            const logData = [team, { cubes: teamCubes }];
            console.log("Enter pressed, opening effect menu");
            console.log(JSON.stringify(logData, null, 2));

            ws.send(JSON.stringify({ WorldConfigRegion: logData }));
          }
          break;
        case "Escape":
          event.preventDefault();
          break;
        case "ArrowUp":
        case "KeyW":
          setMoveForward(true);
          break;
        case "ArrowLeft":
        case "KeyA":
          setMoveLeft(true);
          break;
        case "ArrowDown":
        case "KeyS":
          setMoveBackward(true);
          break;
        case "ArrowRight":
        case "KeyD":
          setMoveRight(true);
          break;
        case "Space":
          setMoveUp(true);
          break;
        case "ShiftLeft":
          setMoveDown(true);
          break;
        case "KeyR":
          resetSelectedCubes();
          break;
        default:
          break;
      }
    },
    [isLocked, exitMovementMode, selectedCubes]
  );

  const resetSelectedCubes = useCallback(() => {
    selectedCubes.forEach((cube) => {
      cube.material.opacity = 0.02;
    });
    setSelectedCubes([]);
  }, [selectedCubes]);

  const handleKeyUp = useCallback(
    (event) => {
      if (!isLocked) return;
      switch (event.code) {
        case "ArrowUp":
        case "KeyW":
          setMoveForward(false);
          break;
        case "ArrowLeft":
        case "KeyA":
          setMoveLeft(false);
          break;
        case "ArrowDown":
        case "KeyS":
          setMoveBackward(false);
          break;
        case "ArrowRight":
        case "KeyD":
          setMoveRight(false);
          break;
        case "Space":
          setMoveUp(false);
          break;
        case "ShiftLeft":
          setMoveDown(false);
          break;
        default:
          break;
      }
    },
    [isLocked]
  );

  useEffect(() => {
    const handleKeyDownListener = (event) => handleKeyDown(event);
    const handleKeyUpListener = (event) => handleKeyUp(event);

    if (isLocked) {
      window.addEventListener("keydown", handleKeyDownListener);
      window.addEventListener("keyup", handleKeyUpListener);
    }

    return () => {
      window.removeEventListener("keydown", handleKeyDownListener);
      window.removeEventListener("keyup", handleKeyUpListener);
    };
  }, [isLocked, handleKeyDown, handleKeyUp]);

  const handleMouseClick = useCallback(
    (event) => {
      if (!isLocked) {
        enterMovementMode(event);
        const raycaster = new THREE.Raycaster();
        const center = new THREE.Vector2(0, 0); // Center of the screen

        if (cameraRef.current && sceneRef.current) {
          raycaster.setFromCamera(center, cameraRef.current);

          const intersects = raycaster.intersectObjects(cubesRef.current, true);
          if (intersects.length > 0) {
            const clickedCube = intersects[0].object;
            const cubePosition = clickedCube.position;

            const cubeSize = 16;
            const x =
              Math.floor(cubePosition.x / cubeSize) * cubeSize + cubeSize / 2;
            const y =
              Math.floor(cubePosition.y / cubeSize) * cubeSize + cubeSize / 2;
            const z =
              Math.floor(cubePosition.z / cubeSize) * cubeSize + cubeSize / 2;

            console.log(`Selected cube center: (${x}, ${y}, ${z})`);
            console.log(
              `Raw cube position: (${cubePosition.x}, ${cubePosition.y}, ${cubePosition.z})`
            );

            if (selectedCubes.includes(clickedCube)) {
              clickedCube.material.opacity = 0.02;
              clickedCube.material.color.setHex(0xffffff);
              setSelectedCubes(
                selectedCubes.filter((cube) => cube !== clickedCube)
              );
            } else {
              clickedCube.material.opacity = 0.1;
              clickedCube.material.color.setHex(0xffffff); // White color for selected cubes
              setSelectedCubes([...selectedCubes, clickedCube]);
            }
          }
        }
      }
    },
    [isLocked, enterMovementMode, selectedCubes]
  );

  useEffect(() => {
    if (initialized && containerRef.current) {
      containerRef.current.addEventListener("click", handleMouseClick);

      return () => {
        if (containerRef.current) {
          containerRef.current.removeEventListener("click", handleMouseClick);
        }
      };
    }
  }, [initialized, handleMouseClick]);

  const handleMovement = useCallback(() => {
    if (!isLocked  || !controlsRef.current) return;

    const currentTime = performance.now();
    const delta = (currentTime - prevTimeRef.current) / 1000;

    velocityRef.current.x -= velocityRef.current.x * 10.0 * delta;
    velocityRef.current.z -= velocityRef.current.z * 10.0 * delta;
    velocityRef.current.y -= velocityRef.current.y * 10.0 * delta;

    directionRef.current.z = Number(moveForward) - Number(moveBackward);
    directionRef.current.x = Number(moveRight) - Number(moveLeft);
    directionRef.current.y = Number(moveUp) - Number(moveDown);
    directionRef.current.normalize();

    const speed = 500.0;
    if (moveForward || moveBackward)
      velocityRef.current.z -= directionRef.current.z * speed * delta;
    if (moveLeft || moveRight)
      velocityRef.current.x -= directionRef.current.x * speed * delta;
    if (moveUp || moveDown)
      velocityRef.current.y += directionRef.current.y * speed * delta;

    controlsRef.current.moveRight(-velocityRef.current.x * delta);
    controlsRef.current.moveForward(-velocityRef.current.z * delta);
    controlsRef.current.getObject().position.y += velocityRef.current.y * delta;

    if (skyRef.current) {
      skyRef.current.position.copy(controlsRef.current.getObject().position);
    }

    prevTimeRef.current = currentTime;
  }, [
    isLocked,
    moveForward,
    moveBackward,
    moveLeft,
    moveRight,
    moveUp,
    moveDown,
  ]);

  const handleAnimationFrame = useCallback(() => {
    handleMovement();
    if (rendererRef.current && sceneRef.current && cameraRef.current) {
      rendererRef.current.render(sceneRef.current, cameraRef.current);
    }
    animationFrameRef.current = requestAnimationFrame(handleAnimationFrame);
  }, [handleMovement]);

  useEffect(() => {
    if (initialized) {
      animationFrameRef.current = requestAnimationFrame(handleAnimationFrame);
    }
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [initialized, handleAnimationFrame]);


  const createCubesBasedOnMinecraftWorld = () => {
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
          const wireframe = new THREE.LineSegments(
            edgesGeometry,
            edgesMaterial
          );
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

  const getTextureType = (textureName) => {
    if (textureName.includes("block")) return "block";
    if (textureName.includes("entity")) return "entity";
    if (textureName.includes("painting")) return "painting";
    if (textureName.includes("banner")) return "banner";
    if (textureName.includes("models")) return "models";
    return "block";
  };

  useEffect(() => {
    if (!initialized) {
      init();
    }
  }, [initialized]);

  useEffect(() => {
    const handlePointerLockChange = () => {
      const isLocked = document.pointerLockElement === containerRef.current;
      console.log(
        "Pointer lock state changed:",
        isLocked ? "locked" : "unlocked"
      );
      setIsLocked(isLocked);
      if (!isLocked) {
        setShowInstructions(true);
        // Reset movement state
      }
    };

    document.addEventListener("pointerlockchange", handlePointerLockChange);
    return () => {
      document.removeEventListener(
        "pointerlockchange",
        handlePointerLockChange
      );
    };
  }, []);

  useEffect(() => {
    if (initialized) {
      document.addEventListener("pointerlockchange", onPointerLockChange);
      document.addEventListener("pointerlockerror", onPointerLockError);

      if (controlsRef.current) {
        sceneRef.current.add(controlsRef.current.getObject());
      }

      handleAnimationFrame();
    }

    return () => {
      document.removeEventListener("pointerlockchange", onPointerLockChange);
      document.removeEventListener("pointerlockerror", onPointerLockError);
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

      containerRef.current.addEventListener("click", handleUserInteraction);

      return () => {
        if (containerRef.current) {
          containerRef.current.removeEventListener(
            "click",
            handleUserInteraction
          );
        }
      };
    }
  }, [initialized, isLocked, enterMovementMode]);

  useEffect(() => {
    setShowCrosshair(isLocked);
  }, [isLocked]);

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", height: "100%", position: "relative" }}
      tabIndex="0"
    >
      {showInstructions && (
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            width: "100%",
            height: "100%",
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            alignItems: "center",
            background: "rgba(0,0,0,0.5)",
            color: "white",
            fontSize: "24px",
            cursor: "pointer",
          }}
        >
          <div>Click to Start</div>
          <div style={{ fontSize: "18px", marginTop: "20px" }}>
            WASD or Arrow keys to move, Space to go up, Shift to go down
            <br />
            Mouse to look around, ESC to exit, Click to select cubes
            <br />R to reset selection, Enter to open effect menu
          </div>
        </div>
      )}

      {showTeamAlert && (
        <div
          style={{
            position: "absolute",
            top: "20px",
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(255,0,0,0.8)",
            color: "white",
            padding: "10px",
            borderRadius: "5px",
            zIndex: 1000,
          }}
        >
          You have to select a team to join before entering movement mode.
        </div>
      )}
      {showCrosshair && (
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: "50%",
            width: "20px",
            height: "20px",
            transform: "translate(-50%, -50%)",
            pointerEvents: "none",
          }}
        >
          <svg width="20" height="20" xmlns="http://www.w3.org/2000/svg">
            <circle cx="10" cy="10" r="2" fill="white" />
            <line
              x1="0"
              y1="10"
              x2="20"
              y2="10"
              stroke="white"
              strokeWidth="2"
            />
            <line
              x1="10"
              y1="0"
              x2="10"
              y2="20"
              stroke="white"
              strokeWidth="2"
            />
          </svg>
        </div>
      )}
    </div>
  );
};

export default ThreeJsScene;
