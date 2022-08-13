import React from "react";
import { Table } from "@devtools-ds/table";
import { ObjectInspector } from "@devtools-ds/object-inspector";
import { Loupedeck } from "../components/Loupedeck";
import { listen } from "@tauri-apps/api/event";
import { EventUpdatePayload } from "../types";
import { Button, Stack } from "rsuite";
import { invoke } from "@tauri-apps/api";

export const EventViewer = () => {
  const [eventList, dispatch] = React.useReducer((state, action) => {
    switch (action.type) {
      case "add":
        return [...state, action.payload];
      case "clear":
        return [];
      default:
        return state;
    }
  }, []);

  React.useEffect(() => {
    const unlisten = listen<EventUpdatePayload>("event-update", (evt) => {
      dispatch({ type: "add", payload: evt });
    });

    return () => {
      unlisten.then((l) => l());
    };
  });

  return (
    <div>
      <Button
        onClick={() => {
          invoke("vibrate");
        }}
      >
        Vibrate
      </Button>
      <Stack direction="row" alignItems="flex-start">
        <div>
          <Loupedeck />
        </div>

        <div
          style={{
            padding: "20px",
          }}
        >
          <Button
            onClick={() => {
              dispatch({ type: "clear" });
            }}
          >
            Clear
          </Button>
          <div
            style={{
              maxHeight: "400px",
              overflow: "auto",
            }}
          >
            <Table>
              <Table.Body>
                {eventList.map((evt, index) => (
                  <Table.Row key={index}>
                    <Table.Cell>
                      <ObjectInspector data={evt} />
                    </Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table>
          </div>
        </div>
      </Stack>
    </div>
  );
};
