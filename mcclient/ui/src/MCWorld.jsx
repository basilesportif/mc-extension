import React, { useEffect, useRef, useState, useCallback } from "react";
import * as THREE from "three";
import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
import { MTLLoader } from "three/examples/jsm/loaders/MTLLoader.js";
import { PointerLockControls } from "three/examples/jsm/controls/PointerLockControls.js";
import * as threeJsSetup from "./utils/threeJsSetup";
import { CubeConstructor } from "./utils/CubeConstructor";
import { Crosshair, Instructions, TeamAlert } from "./components/MCWorldElements";
import  EffectMenu  from "./components/EffectMenu";
import InfoPanel from './components/InfoPanel';

const ThreeJsScene = ({ ws, ourInTeam, lobby, setPaintCubesWithData }) => {
  const sceneRef = useRef(null);
  const cameraRef = useRef(null);
  const rendererRef = useRef(null);
  const controlsRef = useRef(null);
  const minecraftWorldRef = useRef(null);
  const cubesRef = useRef([]);
  const animationFrameRef = useRef(null);
  const containerRef = useRef(null);
  const skyRef = useRef(null);

  const [initialized, setInitialized] = useState(false);
  const [isLocked, setIsLocked] = useState(false);
  const [showInstructions, setShowInstructions] = useState(true);
  const [shouldLoadWorld, setShouldLoadWorld] = useState(true);
  const [areCubesSelectable, setAreCubesSelectable] = useState(false);

  const [moveForward, setMoveForward] = useState(false);
  const [moveBackward, setMoveBackward] = useState(false);
  const [moveLeft, setMoveLeft] = useState(false);
  const [moveRight, setMoveRight] = useState(false);
  const [moveUp, setMoveUp] = useState(false);
  const [moveDown, setMoveDown] = useState(false);

  const [showEffectMenu, setShowEffectMenu] = useState(false);
  const [selectedEffects, setSelectedEffects] = useState([]);

  const effectOptions = ['Slowness', 'Weakness', 'Nausea', 'Hunger', 'Blindness', 'Poison', 'Wither', 'Levitation'];

  const velocityRef = useRef(new THREE.Vector3());
  const directionRef = useRef(new THREE.Vector3());
  const prevTimeRef = useRef(performance.now());

  const [selectedCubes, setSelectedCubes] = useState([]);

  const [showTeamAlert, setShowTeamAlert] = useState(false);
  const [showCrosshair, setShowCrosshair] = useState(false);

  const [storedPlayerPosition, setStoredPlayerPosition] = useState(null);
  const [storedCameraDirection, setStoredCameraDirection] = useState(null);

  const [unpaintedCubesCount, setUnpaintedCubesCount] = useState(0);
  const [team1CubesCount, setTeam1CubesCount] = useState(0);
  const [team2CubesCount, setTeam2CubesCount] = useState(0);

  const init = async () => {
    if (sceneRef.current) {
      console.log("Scene already initialized.");
      return;
    }

    try {
      console.log("Initializing Three.js scene...");
      threeJsSetup.setupThreeJsScene(sceneRef, cameraRef, rendererRef, containerRef, skyRef);
  
      console.log("Loading Minecraft world...");
      await loadMinecraftWorld();
  
      console.log("Three.js scene initialized.");
      setInitialized(true);
    } catch (error) {
      console.error("Error initializing scene:", error);
    }
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
                  child.frustumCulled = false;
                  if (child.material.map) {
                    const textureType = getTextureType(child.material.map.name);
                    console.log(`Texture type: ${textureType}`);
                  }
                }
              });
              sceneRef.current.add(object);
              minecraftWorldRef.current = object;
              console.log("Minecraft world added to scene.");
              CubeConstructor(minecraftWorldRef, sceneRef, cubesRef);
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

  useEffect(() => {
    const handleLock = () => {
      setShowInstructions(false);
      setIsLocked(true); 
    };

    const handleUnlock = () => {
      if (showEffectMenu) {
        setShowInstructions(false);
        setAreCubesSelectable(false);
      } else {
        setShowInstructions(true);
      }
      setIsLocked(false);
    };

    const TrackPointerLockControls = () => {
      if (cameraRef.current && containerRef.current && initialized) {
        const controls = new PointerLockControls(
          cameraRef.current,
          containerRef.current
        );
        controlsRef.current = controls;

        controls.addEventListener("lock", handleLock);
        controls.addEventListener("unlock", handleUnlock);

        sceneRef.current.add(controls.getObject());
      }
    };

    if (initialized) {
      TrackPointerLockControls();
    }

    return () => {
      if (controlsRef.current) {
        const controls = controlsRef.current;
        controls.removeEventListener("lock", handleLock);
        controls.removeEventListener("unlock", handleUnlock);
      }
    };
  }, [initialized, showEffectMenu]);


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
        setAreCubesSelectable(true);
        
        // Restore the player's position and camera direction
        if (storedPlayerPosition && storedCameraDirection) {
          controlsRef.current.getObject().position.copy(storedPlayerPosition);
          cameraRef.current.getWorldDirection(storedCameraDirection);
        }
      } else {
        // Handle case when pointer is already locked
        console.log("Pointer is already locked.");
      }
    },
    [ourInTeam, storedPlayerPosition, storedCameraDirection]
  );

  const exitMovementMode = useCallback(() => {
    if (controlsRef.current) {
      document.exitPointerLock();
      // Store the player's position and camera direction
      const playerPosition = controlsRef.current.getObject().position.clone();
      const cameraDirection = cameraRef.current.getWorldDirection(new THREE.Vector3());

      setStoredPlayerPosition(playerPosition);
      setStoredCameraDirection(cameraDirection);
    }
    
    setShouldLoadWorld(false);
    setAreCubesSelectable(false);
    setMoveForward(false);
    setMoveBackward(false);
    setMoveLeft(false);
    setMoveRight(false);
    setMoveUp(false);
    setMoveDown(false);
  }, []);

  const handleKeyDown = useCallback(
    (event) => {
      if (!isLocked) return;
      switch (event.code) {
        case "Enter":
          if (controlsRef.current && document.pointerLockElement === containerRef.current) {
            document.exitPointerLock();
          }
          setShowEffectMenu(true);
          setShowInstructions(false);
          break;
        case "Escape":
          console.log("Escape key pressed");
          exitMovementMode();
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
    console.log("Resetting selected cubes");
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
      if (!isLocked && !showEffectMenu) {
        //console.log("Entering movement mode");
        //console.log("Is locked:", isLocked);
        enterMovementMode(event);
      }
      
      if (areCubesSelectable && !showEffectMenu) {
        //console.log("Are cubes selectable:", areCubesSelectable);
        //console.log("Is locked:", isLocked);
        const raycaster = new THREE.Raycaster();
        const center = new THREE.Vector2(0, 0); // Center of the screen

        if (cameraRef.current && sceneRef.current) {
          raycaster.setFromCamera(center, cameraRef.current);

          const intersects = raycaster.intersectObjects(cubesRef.current, true);
          if (intersects.length > 0) {
            const clickedCube = intersects[0].object;
            const cubePosition = clickedCube.position;
            console.log(
              `Raw cube position: (${cubePosition.x}, ${cubePosition.y}, ${cubePosition.z})`
            );

            if (selectedCubes.includes(clickedCube)) {
              clickedCube.material.opacity = 0.001;
              clickedCube.material.color.setHex(0xffffff);
              setSelectedCubes(
                selectedCubes.filter((cube) => cube !== clickedCube)
              );
            } else {
              clickedCube.material.opacity = 0.4;
              clickedCube.material.color.setHex(0x808080); // White color for selected cubes
              setSelectedCubes([...selectedCubes, clickedCube]);
            }
          }
        }
      }
      

    },
    [isLocked, enterMovementMode, selectedCubes, areCubesSelectable]
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
    if (!isLocked || !controlsRef.current) return;

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

    // Restrict the camera's movement within a bounding box (check beforehand whether it fits)
    const boundingBoxPadding = 50; // Adjust this value to change the padding around the Minecraft world
    if (minecraftWorldRef.current && cubesRef.current) {
      const bbox = new THREE.Box3().setFromObject(minecraftWorldRef.current);
      const size = bbox.getSize(new THREE.Vector3());
      const center = bbox.getCenter(new THREE.Vector3());

      const minX = center.x - size.x / 2 - boundingBoxPadding;
      const maxX = center.x + size.x / 2 + boundingBoxPadding;
      const minY = 0; // Assuming the ground is at y = 0
      const maxY = center.y + size.y / 2 + boundingBoxPadding;
      const minZ = center.z - size.z / 2 - boundingBoxPadding;
      const maxZ = center.z + size.z / 2 + boundingBoxPadding;

      const cameraPosition = controlsRef.current.getObject().position;
      cameraPosition.clamp(
        new THREE.Vector3(minX, minY, minZ),
        new THREE.Vector3(maxX, maxY, maxZ)
      );
    }

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
    if (initialized) {
      if (controlsRef.current) {
        sceneRef.current.add(controlsRef.current.getObject());
      }
      handleAnimationFrame();
    }
    return () => {
      cancelAnimationFrame(animationFrameRef.current);
    };
  }, [initialized]);


  useEffect(() => {
    setShowCrosshair(isLocked);
  }, [isLocked]);

  const paintCubes = useCallback(() => {
    const { world_config, goal_post, team1, team2 } = lobby;
    let unpaintedCount = 0;
    let team1Count = 0;
    let team2Count = 0;

    if (world_config) {
      cubesRef.current.forEach((cube) => {
        const cubePosition = cube.position;
        const cubeSize = 16;
        const x = Math.floor(cubePosition.x / cubeSize) * cubeSize + cubeSize / 2;
        const y = Math.floor(cubePosition.y / cubeSize) * cubeSize + cubeSize / 2;
        const z = Math.floor(cubePosition.z / cubeSize) * cubeSize + cubeSize / 2;
        const cubeCenter = { center: [x, y, z], side_length: cubeSize };

        // Check if the cube is the goalpost
        const isGoalpost = JSON.stringify(cubeCenter) === JSON.stringify(goal_post);

        // Check if the cube is in Team 1's spawn point
        const isTeam1SpawnPoint = JSON.stringify(cubeCenter) === JSON.stringify(team1.spawn_point);

        // Check if the cube is in Team 2's spawn point
        const isTeam2SpawnPoint = JSON.stringify(cubeCenter) === JSON.stringify(team2.spawn_point);

        if (isGoalpost) {
          cube.material.color.setHex(0x800080); // Purple color for the goalpost
          cube.material.opacity = 0.5;
        } else if (isTeam1SpawnPoint) {
          cube.material.color.setHex(0xffff00); // Yellow color for Team 1's spawn point
          cube.material.opacity = 0.5;
        } else if (isTeam2SpawnPoint) {
          cube.material.color.setHex(0x00ffff); // Cyan color for Team 2's spawn point
          cube.material.opacity = 0.5;
        } else {
          const team1Config = team1.cubes?.find(([cubeConfig]) => {
            return JSON.stringify(cubeConfig) === JSON.stringify(cubeCenter);
          });

          const team2Config = team2.cubes?.find(([cubeConfig]) => {
            return JSON.stringify(cubeConfig) === JSON.stringify(cubeCenter);
          });

          if (team1Config && team2Config) {
            cube.material.color.setHex(0xff0000); // Red for double effects
            cube.material.opacity = 0.2;
          } else if (team1Config) {
            cube.material.color.setHex(0x0000ff); // Blue for Team 1
            cube.material.opacity = 0.2;
            team1Count++;
          } else if (team2Config) {
            cube.material.color.setHex(0x00ff00); // Green for Team 2
            cube.material.opacity = 0.2;
            team2Count++;
          } else {
            cube.material.color.setHex(0xffffff); // White for unpainted cubes
            cube.material.opacity = 0.0001;
            unpaintedCount++;
          }
        }
      });
    }

    setUnpaintedCubesCount(unpaintedCount);
    setTeam1CubesCount(team1Count);
    setTeam2CubesCount(team2Count);
  }, [lobby.world_config, lobby.goal_post, lobby.team1, lobby.team2]);

  useEffect(() => {
    console.log('lobby changed:', lobby);
  }, [lobby]);

  useEffect(() => {
    paintCubes();
  }, [paintCubes, lobby]);

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", height: "100%", position: "relative" }}
      tabIndex="0"
    >
      {showInstructions && <Instructions />}
      {showTeamAlert && <TeamAlert />}
      {showCrosshair && <Crosshair />}
        {showEffectMenu && <EffectMenu
          showEffectMenu={showEffectMenu}
          effectOptions={effectOptions}
          selectedEffects={selectedEffects}
          setSelectedEffects={setSelectedEffects}
          ourInTeam={ourInTeam}
          ws={ws}
          controlsRef={controlsRef}
          containerRef={containerRef}
          setAreCubesSelectable={setAreCubesSelectable}
          selectedCubes={selectedCubes}
          setShowEffectMenu={setShowEffectMenu}
          lobby={lobby}
          paintCubes={paintCubes}
         />}
      <InfoPanel
        unpaintedCubesCount={unpaintedCubesCount}
        team1CubesCount={team1CubesCount}
        team2CubesCount={team2CubesCount}
      />
    </div>
  );
};

export default ThreeJsScene;
