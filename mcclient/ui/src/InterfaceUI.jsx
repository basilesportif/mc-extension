import React from 'react';
import App from './App';
import ThreeJsScene from './MCWorld';

const InterfaceUI = () => {
  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <div style={{ width: '30%', padding: '20px', borderRight: '1px solid #ccc', overflowY: 'auto' }}>
        <App />
      </div>
      <div style={{ width: '70%', position: 'relative' }}>
        <ThreeJsScene />
      </div>
    </div>
  );
};

export default InterfaceUI;