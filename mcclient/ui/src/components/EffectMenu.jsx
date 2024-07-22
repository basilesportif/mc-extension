import { useRef, useEffect } from 'react';

const EffectMenu = ({
  showEffectMenu,
  effectOptions,
  selectedEffects,
  setSelectedEffects,
  ourInTeam,
  ws,
  controlsRef,
  containerRef,
  setAreCubesSelectable,
  selectedCubes,
  setShowEffectMenu,
  lobby,
  paintCubes,
}) => {
  if (!showEffectMenu) return null;

  const handleApplyEffects = () => {
    if (selectedCubes.length > 0) {
      // the exact format which backend needs
      const teamCubes = selectedCubes.reduce((acc, cube) => {
        const center = [
          cube.position.x,
          cube.position.y,
          cube.position.z,
        ];
        const side_length = 16; // Assuming a fixed side length
        const element = [{ center, side_length }, [selectedEffects]];
        // if cube in list, dont add
        if (
          !acc.some(
            (e) =>
              e[0].center[0] === center[0] &&
              e[0].center[1] === center[1] &&
              e[0].center[2] === center[2] &&
              e[0].side_length === side_length
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

      // Send the updated lobby state over WebSocket
      ws.send(JSON.stringify({ WorldConfigRegion: logData }));

    }

    setShowEffectMenu(false);
    setSelectedEffects([]);
    // Relock the pointer to the Minecraft world
    if (controlsRef.current && containerRef.current) {
      containerRef.current.requestPointerLock();
      setAreCubesSelectable(true);
    }
  };

  const handleEffectSelection = (effect) => {
    setSelectedEffects((prevEffects) =>
      prevEffects.includes(effect)
        ? prevEffects.filter((e) => e !== effect)
        : [...prevEffects, effect]
    );
  };

  const handleCancelEffects = () => {
    console.log('Effect selection canceled');
    setShowEffectMenu(false);
    setSelectedEffects([]);
    if (controlsRef.current && containerRef.current) {
      containerRef.current.requestPointerLock();
      setAreCubesSelectable(true);
    }
  };

  return (
    <div
      style={{
        position: 'absolute',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        color: 'white',
        padding: '20px',
        borderRadius: '5px',
        zIndex: 1000,
      }}
    >
      <h2>Select Effects</h2>
      {effectOptions.map((effect) => (
        <div key={effect}>
          <label>
            <input
              type="checkbox"
              checked={selectedEffects.includes(effect)}
              onChange={() => handleEffectSelection(effect)}
            />
            {effect}
          </label>
        </div>
      ))}
      <div style={{ marginTop: '20px' }}>
        <button onClick={handleApplyEffects}>Apply</button>
        <button onClick={handleCancelEffects} style={{ marginLeft: '10px' }}>
          Cancel
        </button>
      </div>
    </div>
  );
};

export default EffectMenu;