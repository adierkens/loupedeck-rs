import { useEffect, useMemo, useState } from "react";
import logo from "./logo.svg";
import "./App.css";
import { invoke } from "@tauri-apps/api";

const list_ld_ports = async (): Promise<string[]> => {
  const ports = await invoke<string[]>("list_ld_ports");
  return ports;
};

import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("state-update", (evt) => {
  console.log("state-update", evt);
});

await listen("event-update", (evt) => {
  console.log("event", evt);
});

function App() {
  const [ports, setPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string | undefined>();

  const refreshPorts = useMemo(() => {
    return () => list_ld_ports().then(setPorts);
  }, [setPorts]);

  useEffect(() => {
    refreshPorts();
  }, []);

  return (
    <div className="App">
      <select
        onChange={(e) => {
          setSelectedPort(e.target.value);
        }}
      >
        <option>Select</option>
        {ports.map((p) => (
          <option key={p} value={p}>
            {p}
          </option>
        ))}
      </select>

      <button
        disabled={selectedPort === undefined}
        onClick={() => {
          invoke("connect_ld", { port: selectedPort });
        }}
      >
        Connect
      </button>

      <button
        onClick={() => {
          refreshPorts();
        }}
      >
        Refresh
      </button>

      <button
        onClick={() => {
          invoke("test_state");
        }}
      >
        Test State
      </button>
    </div>
  );
}

export default App;
