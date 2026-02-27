#!/usr/bin/env node

import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function runCommand(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    encoding: "utf8",
    stdio: options.capture ? ["ignore", "pipe", "pipe"] : "inherit",
    env: options.env ?? process.env,
    timeout: options.timeoutMs ?? 0,
  });
  return result;
}

function resolveAdbCommand() {
  const sdkRoot = process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT;
  const adbName = process.platform === "win32" ? "adb.exe" : "adb";
  const candidates = [];

  if (sdkRoot) {
    candidates.push(path.join(sdkRoot, "platform-tools", adbName));
  }
  candidates.push(path.join(process.env.HOME || "", "Library/Android/sdk/platform-tools", adbName));
  candidates.push("adb");

  return candidates.find((candidate) => candidate === "adb" || existsSync(candidate)) || "adb";
}

function parseDevices(adbOutput) {
  return adbOutput
    .split("\n")
    .slice(1)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [serial, state] = line.split(/\s+/, 3);
      return { serial, state };
    });
}

function getConnectedDevices(adb) {
  const devicesResult = runCommand(adb, ["devices"], { capture: true });
  return parseDevices(devicesResult.stdout || "");
}

function pickReadyDevice(devices) {
  const preferredSerial = process.env.RIGHT_HAND_ANDROID_SERIAL?.trim();
  if (preferredSerial) {
    const preferred = devices.find((d) => d.serial === preferredSerial && d.state === "device");
    if (preferred) {
      return preferred;
    }
  }
  return devices.find((d) => d.state === "device");
}

function formatDevices(devices) {
  if (!devices.length) {
    return "<none>";
  }
  return devices.map((d) => `${d.serial}:${d.state}`).join(", ");
}

function getBootCompleted(adb, serial) {
  const result = runCommand(adb, ["-s", serial, "shell", "getprop", "sys.boot_completed"], {
    capture: true,
  });
  if (result.error || result.status !== 0) {
    return "";
  }
  return (result.stdout || "").trim();
}

function hasPackagesOutput(text) {
  return typeof text === "string" && text.includes("package:");
}

function isPackageServiceReady(adb, serial) {
  // Prefer `cmd package`, but fallback to `pm` for images where cmd service routing is flaky.
  const cmdResult = runCommand(
    adb,
    ["-s", serial, "shell", "cmd", "package", "list", "packages"],
    { capture: true, timeoutMs: 8000 }
  );
  if (!cmdResult.error && cmdResult.status === 0 && hasPackagesOutput(cmdResult.stdout || "")) {
    return true;
  }

  const pmResult = runCommand(
    adb,
    ["-s", serial, "shell", "pm", "list", "packages"],
    { capture: true, timeoutMs: 8000 }
  );
  if (!pmResult.error && pmResult.status === 0 && hasPackagesOutput(pmResult.stdout || "")) {
    return true;
  }

  return false;
}

async function waitForPackageServiceReady(adb, serial, timeoutMs = 120_000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (isPackageServiceReady(adb, serial)) {
      return true;
    }
    await delay(2000);
  }
  return false;
}

function resolveEmulatorCommand() {
  const sdkRoot = process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT;
  const emulatorName = process.platform === "win32" ? "emulator.exe" : "emulator";
  const candidates = [];

  if (sdkRoot) {
    candidates.push(path.join(sdkRoot, "emulator", emulatorName));
  }
  candidates.push(path.join(process.env.HOME || "", "Library/Android/sdk/emulator", emulatorName));
  candidates.push("emulator");

  return candidates.find((candidate) => candidate === "emulator" || existsSync(candidate)) || "emulator";
}

function listAvailableAvds(emulatorCommand) {
  const result = runCommand(emulatorCommand, ["-list-avds"], { capture: true });
  if (result.error || result.status !== 0) {
    return [];
  }
  return (result.stdout || "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function avdScore(avdName) {
  const name = avdName.toLowerCase();
  let score = 0;

  if (name.includes("api_35") || name.includes("api35")) score += 50;
  if (name.includes("google") || name.includes("play")) score += 20;
  if (name.includes("x86_64") || name.includes("x64")) score += 10;
  if (name.includes("api_36.1") || name.includes("preview")) score -= 30;

  return score;
}

function pickBestAvd(avdCandidates) {
  if (!avdCandidates.length) {
    return undefined;
  }
  return [...avdCandidates]
    .sort((a, b) => {
      const scoreDiff = avdScore(b) - avdScore(a);
      if (scoreDiff !== 0) {
        return scoreDiff;
      }
      return a.localeCompare(b);
    })[0];
}

function startEmulator(emulatorCommand, avdName) {
  const child = spawn(emulatorCommand, [`@${avdName}`], {
    detached: true,
    stdio: "ignore",
  });
  child.unref();
}

async function waitForBootCompleted(adb, serial, timeoutMs = 180_000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const bootCompleted = getBootCompleted(adb, serial);
    if (bootCompleted === "1") {
      return true;
    }
    await delay(2000);
  }
  return false;
}

function sanitizeAndroidGradleSettings() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const projectRoot = path.resolve(scriptDir, "..");
  const candidates = [
    path.join(process.cwd(), "src-tauri", "gen", "android", "settings.gradle"),
    path.join(process.cwd(), "gen", "android", "settings.gradle"),
    path.join(projectRoot, "src-tauri", "gen", "android", "settings.gradle"),
  ];
  const uniqueCandidates = [...new Set(candidates)];

  for (const settingsPath of uniqueCandidates) {
    if (!existsSync(settingsPath)) {
      continue;
    }

    const content = readFileSync(settingsPath, "utf8");
    const sanitized = content.replace(
      /<ydef\s+tauriSettings\s*=\s*file\('tauri\.settings\.gradle'\)/g,
      "def tauriSettings = file('tauri.settings.gradle')"
    );

    if (sanitized !== content) {
      writeFileSync(settingsPath, sanitized, "utf8");
      console.warn(`[tauri] Repaired malformed Gradle settings line at ${settingsPath}.`);
    }
  }
}

function ensureAndroidStudioServerAddrFile() {
  const tauriConfigCandidates = [
    path.join(process.cwd(), "tauri.conf.json"),
    path.join(process.cwd(), "src-tauri", "tauri.conf.json"),
  ];
  const tauriConfigPath = tauriConfigCandidates.find((candidate) => existsSync(candidate));
  if (!tauriConfigPath) {
    return;
  }

  let config;
  try {
    config = JSON.parse(readFileSync(tauriConfigPath, "utf8"));
  } catch (error) {
    console.warn(`[tauri] Failed to parse ${tauriConfigPath}: ${error.message}`);
    return;
  }

  const identifier = config?.identifier;
  if (typeof identifier !== "string" || identifier.trim().length === 0) {
    return;
  }

  const overrideAddr = process.env.RIGHT_HAND_DEV_SERVER_ADDR?.trim();
  const devUrl = config?.build?.devUrl;
  let resolvedAddr = overrideAddr || "";
  if (!resolvedAddr && typeof devUrl === "string" && devUrl.trim().length > 0) {
    try {
      const parsed = new URL(devUrl);
      if (parsed.hostname && parsed.port) {
        resolvedAddr = `${parsed.hostname}:${parsed.port}`;
      }
    } catch {
      // Keep fallback if URL parsing fails.
    }
  }
  if (!resolvedAddr) {
    resolvedAddr = "127.0.0.1:1420";
  }

  const addrFilePath = path.join(os.tmpdir(), `${identifier}-server-addr`);
  try {
    const current = existsSync(addrFilePath) ? readFileSync(addrFilePath, "utf8").trim() : "";
    if (current !== resolvedAddr) {
      writeFileSync(addrFilePath, `${resolvedAddr}\n`, "utf8");
      console.log(`[tauri] Prepared Android Studio server addr file: ${addrFilePath}`);
    }
  } catch (error) {
    console.warn(`[tauri] Unable to prepare Android Studio server addr file: ${error.message}`);
  }
}

async function preflightAndroidAdb() {
  const adb = resolveAdbCommand();
  const version = runCommand(adb, ["version"], { capture: true });
  if (version.error || version.status !== 0) {
    console.warn(
      `[tauri] Skipping ADB preflight (adb unavailable at '${adb}'). Android run may fail.`
    );
    return;
  }

  const hasOffline = (devices) => devices.some((d) => d.state === "offline");
  const hasUnauthorized = (devices) => devices.some((d) => d.state === "unauthorized");
  const hasAnyDevice = (devices) => devices.length > 0;

  const recoverAdb = async () => {
    runCommand(adb, ["reconnect", "offline"]);
    runCommand(adb, ["reconnect"]);
    await delay(1200);
    runCommand(adb, ["kill-server"]);
    await delay(500);
    runCommand(adb, ["start-server"]);
    await delay(1200);
  };

  runCommand(adb, ["start-server"]);
  let devices = getConnectedDevices(adb);
  let readyDevice = pickReadyDevice(devices);

  if (hasOffline(devices) || hasUnauthorized(devices) || !readyDevice) {
    console.warn(`[tauri] Initial adb device state: ${formatDevices(devices)}. Running recovery...`);
    for (let attempt = 1; attempt <= 3; attempt += 1) {
      await recoverAdb();
      devices = getConnectedDevices(adb);
      readyDevice = pickReadyDevice(devices);
      if (readyDevice) {
        break;
      }
      if (hasUnauthorized(devices)) {
        console.warn(
          `[tauri] Device is unauthorized (${formatDevices(devices)}). Unlock emulator/device and accept USB debugging prompt.`
        );
      } else {
        console.warn(`[tauri] Recovery attempt ${attempt}/3 did not yield online device: ${formatDevices(devices)}`);
      }
    }
  }

  if (!readyDevice) {
    const emulatorCommand = resolveEmulatorCommand();
    const avdCandidates = listAvailableAvds(emulatorCommand);
    const preferredAvd = process.env.RIGHT_HAND_ANDROID_AVD?.trim();
    const selectedAvd = preferredAvd && avdCandidates.includes(preferredAvd)
      ? preferredAvd
      : pickBestAvd(avdCandidates);

    if (selectedAvd) {
      console.log(`[tauri] No ready Android device found; starting AVD '${selectedAvd}'...`);
      startEmulator(emulatorCommand, selectedAvd);

      const maxWaitMs = 300_000;
      const startTime = Date.now();
      while (Date.now() - startTime < maxWaitMs) {
        await delay(2000);
        runCommand(adb, ["reconnect", "offline"]);
        runCommand(adb, ["start-server"]);
        devices = getConnectedDevices(adb);
        readyDevice = pickReadyDevice(devices);
        if (readyDevice) {
          break;
        }
      }
    }
  }

  if (!readyDevice) {
    console.error(
      `[tauri] No online Android device found after recovery. Current state: ${formatDevices(devices)}.\n` +
      `[tauri] Try in Android Studio Device Manager: Stop emulator -> Cold Boot Now -> wait for home screen -> rerun.`
    );
    process.exit(1);
  }

  if (readyDevice && readyDevice.serial.startsWith("emulator-")) {
    const booted = await waitForBootCompleted(adb, readyDevice.serial, 240_000);
    if (!booted) {
      console.error(
        `[tauri] Emulator ${readyDevice.serial} did not finish booting in time (sys.boot_completed != 1). Cold boot the emulator and rerun.`
      );
      process.exit(1);
    }

    const packageReady = await waitForPackageServiceReady(adb, readyDevice.serial, 120_000);
    if (!packageReady) {
      console.error(
        `[tauri] Emulator ${readyDevice.serial} booted but package manager is not ready.\n` +
        `[tauri] This emulator image/snapshot is unhealthy for app install. Use Device Manager -> Cold Boot Now or Wipe Data.\n` +
        `[tauri] If it persists, recreate AVD with a stable Google APIs/Google Play image (recommended API 35 x86_64).`
      );
      process.exit(1);
    }
  }

  if (!hasAnyDevice(devices)) {
    console.error("[tauri] adb reports no devices. Start an emulator from Android Studio Device Manager and rerun.");
    process.exit(1);
  }
}

const isAndroidCommand = args[0] === "android";
const androidSubcommand = args[1];
const isHelpOrVersion = args.includes("-h") || args.includes("--help") || args.includes("--version") || args.includes("-V");
const requiresAdbPreflight = isAndroidCommand
  && androidSubcommand === "dev"
  && !isHelpOrVersion;

if (isAndroidCommand) {
  sanitizeAndroidGradleSettings();
}

if (isAndroidCommand && androidSubcommand === "android-studio-script" && !isHelpOrVersion) {
  ensureAndroidStudioServerAddrFile();
}

if (requiresAdbPreflight) {
  await preflightAndroidAdb();
}

const command = process.platform === "win32" ? "npx.cmd" : "npx";
const child = spawn(command, ["tauri", ...args], {
  stdio: "inherit",
  env: {
    ...process.env,
    CARGO_AUTO_CLEAN_FREQUENCY: process.env.CARGO_AUTO_CLEAN_FREQUENCY ?? "never",
    RUSTUP_TOOLCHAIN: process.env.RUSTUP_TOOLCHAIN ?? "1.92.0",
  },
});

const forwardSignal = (signal) => {
  if (!child.killed) {
    child.kill(signal);
  }
};

process.on("SIGINT", () => forwardSignal("SIGINT"));
process.on("SIGTERM", () => forwardSignal("SIGTERM"));
process.on("SIGHUP", () => forwardSignal("SIGHUP"));

child.on("error", (error) => {
  console.error(`[tauri] Failed to start CLI: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.exit(0);
  }
  process.exit(code ?? 0);
});
