import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import "./index.css";
import App from "./App.tsx";
import { useMailStore } from "./stores/mailStore.ts";
import { useAccountStore } from "./stores/accountStore.ts";

// Check connection status on startup, then load data if connected
useAccountStore.getState().checkConnection().then(() => {
  const { connected } = useAccountStore.getState();
  if (connected) {
    useMailStore.getState().fetchMailboxes();
    useAccountStore.getState().fetchAccountEmail();
  }
});

// Listen for Rust backend events
listen("new-email", () => {
  useMailStore.getState().refresh();
});

listen("emails-updated", () => {
  useMailStore.getState().refresh();
  // Also refresh account email (discovered after first sync)
  useAccountStore.getState().fetchAccountEmail();
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
