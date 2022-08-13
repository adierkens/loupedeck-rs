import React, { useEffect, useMemo, useState } from "react";
import { Button, SelectPicker } from "rsuite";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

interface DeviceConnectionEvent {
  status: "Connected" | "Disconnected" | "Connecting";
}

const list_ld_ports = async (): Promise<string[]> => {
  const ports = await invoke<string[]>("list_ld_ports");
  return ports;
};

const get_connection_status = async () => {
  const status = await invoke<DeviceConnectionEvent["status"]>(
    "get_connection_status"
  );
  return status;
};

const usePorts = () => {
  const [ports, setPorts] = useState<string[]>([]);

  const refreshPorts = useMemo(() => {
    return () => list_ld_ports().then(setPorts);
  }, [setPorts]);

  useEffect(() => {
    refreshPorts();
  }, []);

  return {
    ports,
    refreshPorts,
  };
};

const useConnectionStatus = () => {
  const [status, setStatus] = useState<
    DeviceConnectionEvent["status"] | undefined
  >();

  React.useEffect(() => {
    if (status === undefined) {
      get_connection_status().then(setStatus);
    }
  }, [status]);

  React.useEffect(() => {
    const unlisten = listen<DeviceConnectionEvent>(
      "device-connection-status",
      (evt) => {
        setStatus(evt.payload.status);
      }
    );

    return () => {
      unlisten.then((l) => l());
    };
  });

  return { status };
};

const NewConnection = () => {
  const { ports, refreshPorts } = usePorts();
  const [selectedPort, setSelectedPort] = useState<string | undefined>();

  return (
    <div>
      <SelectPicker
        searchable={false}
        cleanable={false}
        onChange={(value) => {
          setSelectedPort(value as string);
        }}
        data={ports.map((p) => ({ label: p, value: p }))}
      />
      <Button
        onClick={() => {
          refreshPorts();
        }}
      >
        Refresh
      </Button>
      <Button
        disabled={selectedPort === undefined}
        onClick={() => {
          invoke("connect_ld", { port: selectedPort });
        }}
      >
        Connect
      </Button>
    </div>
  );
};

export const Connect = () => {
  const { status } = useConnectionStatus();

  return (
    <div>
      <div>{status}</div>
      {status === "Disconnected" && <NewConnection />}
    </div>
  );
};
