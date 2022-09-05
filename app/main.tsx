import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes, Navigate } from "react-router-dom";
import { DeviceSelection } from "./pages/DeviceSelection";
import { DeviceConfig } from "./pages/DeviceConfig";
import { SideNav } from "./components/Sidenav";
import App from "./App";
import "./index.css";
import { Settings } from "./pages/Settings";
import { ThemeProvider } from "@devtools-ds/themes";
import "rsuite/dist/rsuite.min.css";
import { CustomProvider, Container, Content } from "rsuite";
import { Plugins } from "./pages/Plugins";
import { EventViewer } from "./pages/EventViewer";
import { Editor } from "./pages/Editor";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider theme="chrome" colorScheme="dark">
      <CustomProvider theme="dark">
        <BrowserRouter>
          <Container>
            <SideNav />

            <Container>
              <Content>
                <Routes>
                  <Route path="/" element={<Navigate to="/event-viewer" />} />

                  <Route
                    path="/device-selection"
                    element={<DeviceSelection />}
                  />
                  <Route path="/device-config" element={<DeviceConfig />} />
                  <Route path="/settings" element={<Settings />} />
                  <Route path="/plugins" element={<Plugins />} />
                  <Route path="/editor" element={<Editor />} />
                  <Route path="/event-viewer" element={<EventViewer />} />
                </Routes>
              </Content>
            </Container>
          </Container>
        </BrowserRouter>
      </CustomProvider>
    </ThemeProvider>
  </React.StrictMode>
);
