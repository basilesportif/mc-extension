// Create a new file called InfoPanel.jsx
import React from 'react';

const InfoPanel = ({ unpaintedCubesCount, team1CubesCount, team2CubesCount }) => {
  return (
    <div
      style={{
        position: 'absolute',
        top: '10px',
        right: '10px',
        backgroundColor: 'rgba(255, 255, 255, 0.8)',
        padding: '10px',
        borderRadius: '5px',
        fontFamily: 'Arial, sans-serif',
        fontSize: '14px',
      }}
    >
      <div>Total Cubes: {unpaintedCubesCount + team1CubesCount + team2CubesCount}</div>
      <div>Unpainted Cubes: {unpaintedCubesCount}</div>
      <div>Team 1 Cubes: {team1CubesCount}</div>
      <div>Team 2 Cubes: {team2CubesCount}</div>
      <div>
        <span style={{ color: 'red' }}>Red</span>: Double effects
      </div>
      <div>
        <span style={{ color: 'blue' }}>Blue</span>: Team 1
      </div>
      <div>
        <span style={{ color: 'green' }}>Green</span>: Team 2
      </div>
      <div>
        <span style={{ color: 'white' }}>White</span>: Unpainted cubes
      </div>
      <div>
        <span style={{ color: 'purple' }}>Purple</span>: Goalpost
      </div>
    </div>
  );
};

export default InfoPanel;