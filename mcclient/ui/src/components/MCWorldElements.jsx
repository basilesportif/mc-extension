export const Crosshair = () => (
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
        <line x1="0" y1="10" x2="20" y2="10" stroke="white" strokeWidth="2" />
        <line x1="10" y1="0" x2="10" y2="20" stroke="white" strokeWidth="2" />
      </svg>
    </div>
  );
  
  export const Instructions = () => (
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
  );

  export const TeamAlert = () => (
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
  );