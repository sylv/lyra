import fs from "node:fs";
import path from "node:path";
import { chromium } from "playwright-core";

function isExecutable(filePath) {
  try {
    fs.accessSync(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function findExecutableInPath(candidate) {
  const envPath = process.env.PATH ?? "";
  for (const dir of envPath.split(path.delimiter)) {
    if (!dir) continue;
    const full = path.join(dir, candidate);
    if (fs.existsSync(full) && isExecutable(full)) {
      return full;
    }
  }
  return null;
}

export function findBrowserExecutable() {
  const override = process.env.LYRA_BROWSER;
  if (override && fs.existsSync(override) && isExecutable(override)) {
    return override;
  }

  const candidates = [
    "chromium",
    "chromium-browser",
    "google-chrome",
    "google-chrome-stable",
    "brave-browser",
    "brave",
  ];

  for (const candidate of candidates) {
    const found = findExecutableInPath(candidate);
    if (found) return found;
  }

  const absoluteCandidates = [
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/brave-browser",
    "/usr/bin/brave",
    "/snap/bin/chromium",
    "/snap/bin/brave",
  ];

  for (const candidate of absoluteCandidates) {
    if (fs.existsSync(candidate) && isExecutable(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    "No local Chromium/Brave executable found. Install chromium or brave, or set LYRA_BROWSER=/path/to/browser."
  );
}

export async function launchBrowser() {
  const executablePath = findBrowserExecutable();
  const headless = process.env.LYRA_HEADFUL ? false : true;
  const slowMo = process.env.LYRA_SLOWMO
    ? Number.parseInt(process.env.LYRA_SLOWMO, 10)
    : 0;

  console.log(`[hls-e2e] Using browser: ${executablePath}`);
  console.log(`[hls-e2e] Headless: ${headless ? "true" : "false"}`);

  return await chromium.launch({
    headless,
    executablePath,
    slowMo: Number.isFinite(slowMo) ? slowMo : 0,
  });
}
