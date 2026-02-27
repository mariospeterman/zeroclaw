#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ANDROID_DIR="$APP_DIR/src-tauri/gen/android"

echo "[android-reset] android dir: $ANDROID_DIR"

if [[ ! -d "$ANDROID_DIR" ]]; then
  echo "[android-reset] missing Android project dir. Run: npm run tauri -- android init --ci --skip-targets-install"
  exit 1
fi

if [[ -x "$ANDROID_DIR/gradlew" ]]; then
  echo "[android-reset] stopping Gradle daemons (best effort)"
  (cd "$ANDROID_DIR" && ./gradlew --stop >/dev/null 2>&1 || true)
fi

echo "[android-reset] killing stuck Gradle processes for this project (best effort)"
PROJECT_REGEX="$ANDROID_DIR/gradlew|org\\.gradle\\.launcher\\.daemon\\.bootstrap\\.GradleDaemon"
PIDS="$(pgrep -f "$PROJECT_REGEX" || true)"
if [[ -n "$PIDS" ]]; then
  # shellcheck disable=SC2086
  kill -9 $PIDS >/dev/null 2>&1 || true
fi

echo "[android-reset] killing stale Kotlin compile daemons (best effort)"
KOTLIN_DAEMONS="$(pgrep -f "org.jetbrains.kotlin.daemon.KotlinCompileDaemon" || true)"
if [[ -n "$KOTLIN_DAEMONS" ]]; then
  # shellcheck disable=SC2086
  kill -9 $KOTLIN_DAEMONS >/dev/null 2>&1 || true
fi

echo "[android-reset] removing stale lock files (best effort)"
LOCKS=(
  "$ANDROID_DIR/.gradle/noVersion/buildLogic.lock"
  "$ANDROID_DIR/.gradle/noVersion/fileHashes.lock"
  "$ANDROID_DIR/.gradle/noVersion/dependencies-accessors.lock"
)
for lock in "${LOCKS[@]}"; do
  rm -f "$lock" 2>/dev/null || true
done

ADB_CANDIDATES=(
  "${ANDROID_HOME:-}/platform-tools/adb"
  "${ANDROID_SDK_ROOT:-}/platform-tools/adb"
  "$HOME/Library/Android/sdk/platform-tools/adb"
)
ADB_BIN=""
for candidate in "${ADB_CANDIDATES[@]}"; do
  if [[ -n "$candidate" && -x "$candidate" ]]; then
    ADB_BIN="$candidate"
    break
  fi
done

if [[ -n "$ADB_BIN" ]]; then
  echo "[android-reset] restarting adb server"
  "$ADB_BIN" kill-server >/dev/null 2>&1 || true
  "$ADB_BIN" start-server >/dev/null 2>&1 || true
  "$ADB_BIN" devices || true
fi

echo "[android-reset] done"
