export interface EventUpdatePayload {
  ButtonPress?: {
    button: string;
    dir: "Up" | "Down";
    tx_id: number;
  };

  KnobRotate?: {
    knob: string;
    value: number;
    tx_id: number;
  };
}

export interface PluginIdentifier {
  plugin_id: string;
  plugin_ref: string;
}

export interface PageConfig {
  name: string;
  screen: Array<[KeyLocation, PluginIdentifier]>;
}

export interface KeyLocation {
  x: number;
  y: number;
}
