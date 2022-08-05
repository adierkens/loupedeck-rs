import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { DeviceSelection } from "./pages/DeviceSelection";
import { DeviceConfig } from "./pages/DeviceConfig";
import { Header } from "./components/Header";
import { AppLayout } from "./components/AppLayout";
import App from "./App";
import "./index.css";
import { Settings } from "./pages/Settings";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ChakraProvider>
      <BrowserRouter>
        <AppLayout>
          <Header />
          <Routes>
            <Route path="/" element={<App />} />
            <Route path="/device-selection" element={<DeviceSelection />} />
            <Route path="/device-config" element={<DeviceConfig />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </AppLayout>
      </BrowserRouter>
    </ChakraProvider>
  </React.StrictMode>
);
