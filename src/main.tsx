import React from "react";
import ReactDOM from "react-dom/client";
import { Toaster } from "react-hot-toast";
import RootApp from "./RootApp";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RootApp />
    <Toaster 
      position="top-right"
      toastOptions={{
        duration: 4000,
        style: {
          background: '#1e2832',
          color: '#ffffff',
          border: '1px solid rgba(0, 255, 136, 0.3)',
          borderRadius: '8px',
          fontFamily: 'Orbitron, monospace',
        },
        success: {
          iconTheme: {
            primary: '#00ff88',
            secondary: '#1e2832',
          },
          style: {
            border: '1px solid rgba(0, 255, 136, 0.5)',
          },
        },
        error: {
          iconTheme: {
            primary: '#ff0055',
            secondary: '#1e2832',
          },
          style: {
            border: '1px solid rgba(255, 0, 85, 0.5)',
          },
          duration: 6000,
        },
        loading: {
          iconTheme: {
            primary: '#ffc107',
            secondary: '#1e2832',
          },
          style: {
            border: '1px solid rgba(255, 193, 7, 0.5)',
          },
        },
      }}
    />
  </React.StrictMode>,
);
