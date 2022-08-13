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
