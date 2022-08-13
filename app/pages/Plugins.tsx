import React from "react";
import { emit } from "@tauri-apps/api/event";
import { Form, ButtonToolbar, Button, Input } from "rsuite";

export const Plugins = () => {
  return (
    <Form
      onSubmit={(checked, event) => {
        const data = new FormData(event.target as HTMLFormElement);
        const path = data.get("path");
        emit("trigger-screen-plugin", "time-plugin");
      }}
    >
      <Form.Group>
        <Form.Control name="path"></Form.Control>
      </Form.Group>

      <Form.Group>
        <ButtonToolbar>
          <Button type="submit" appearance="primary">
            Submit
          </Button>
        </ButtonToolbar>
      </Form.Group>
    </Form>
  );
};
