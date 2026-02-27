#!/usr/bin/env node

import net from "node:net";
import { spawn } from "node:child_process";

const HOST = "127.0.0.1";
const PORT = 1420;
const tauriDevHost = process.env.TAURI_DEV_HOST?.trim();

function isPortOpen(host, port, timeoutMs = 800) {
  return new Promise((resolve) => {
    const socket = new net.Socket();
    let settled = false;

    const finish = (value) => {
      if (!settled) {
        settled = true;
        socket.destroy();
        resolve(value);
      }
    };

    socket.setTimeout(timeoutMs);
    socket.once("connect", () => finish(true));
    socket.once("timeout", () => finish(false));
    socket.once("error", () => finish(false));
    socket.connect(port, host);
  });
}

function keepAliveUntilKilled() {
  const timer = setInterval(() => {}, 1 << 30);
  const shutdown = () => {
    clearInterval(timer);
    process.exit(0);
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
  process.on("SIGHUP", shutdown);
}

const portInUse = await isPortOpen(HOST, PORT);
if (portInUse) {
  console.log(`[dev:tauri] Reusing existing frontend server on http://localhost:${PORT}`);
  process.stdin.resume();
  keepAliveUntilKilled();
} else {
  console.log(`[dev:tauri] Starting Vite dev server on http://localhost:${PORT}`);
  const command = process.platform === "win32" ? "npx.cmd" : "npx";
  const args = ["vite", "--port", String(PORT), "--strictPort"];
  if (tauriDevHost) {
    args.push("--host", tauriDevHost);
  }
  const child = spawn(command, args, {
    stdio: "inherit",
  });

  const forward = (signal) => {
    if (!child.killed) {
      child.kill(signal);
    }
  };

  process.on("SIGINT", () => forward("SIGINT"));
  process.on("SIGTERM", () => forward("SIGTERM"));
  process.on("SIGHUP", () => forward("SIGHUP"));

  child.on("error", (error) => {
    console.error(`[dev:tauri] Failed to start Vite: ${error.message}`);
    process.exit(1);
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.exit(0);
    }
    process.exit(code ?? 0);
  });
}
