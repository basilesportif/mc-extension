import React, { useState } from 'react';
import App from './App';
import ThreeJsScene from './MCWorld';

const InterfaceUI = () => {
  const [ourInTeam, setOurInTeam] = useState(null);
  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <div style={{ width: '30%', padding: '20px', borderRight: '1px solid #ccc', overflowY: 'auto' }}>
        <App ourInTeam={ourInTeam} setOurInTeam={setOurInTeam} />
      </div>
      <div style={{ width: '70%', position: 'relative' }}>
        <ThreeJsScene ourInTeam={ourInTeam} />
      </div>
    </div>
  );
};

export default InterfaceUI;