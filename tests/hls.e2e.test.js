import { afterAll, beforeAll, expect, test } from "bun:test";
import { spawn } from "node:child_process";
import fs from "node:fs";
import { setTimeout as delay } from "node:timers/promises";
import path from "node:path";
import puppeteer from "puppeteer";

const SERVER_ROOT = "http://127.0.0.1:4422";
const SERVER_URL = `${SERVER_ROOT}/index.m3u8`;
const PROJECT_ROOT = path.resolve(import.meta.dir, "..");
const INPUT_FILE = path.join(PROJECT_ROOT, "test.mkv");
const HLS_SCRIPT = path.join(
  PROJECT_ROOT,
  "node_modules",
  "hls.js",
  "dist",
  "hls.min.js"
);

let serverProcess;
const diagnostics = {
  serverStdout: [],
  serverStderr: [],
  pageConsole: [],
  pageErrors: [],
};

function logStep(message) {
  console.log(`[hls-test] ${message}`);
}

function findBrowserExecutable() {
  if (process.env.PUPPETEER_EXECUTABLE_PATH) {
    return process.env.PUPPETEER_EXECUTABLE_PATH;
  }

  const candidates = [
    // Brave
    "/usr/bin/brave-browser",
    "/usr/bin/brave",
    "/opt/brave.com/brave/brave",
    // Chromium
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    // Chrome
    "/usr/bin/google-chrome",
    "/opt/google/chrome/chrome",
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

async function waitForServerReady(url, timeoutMs = 30000) {
  const start = Date.now();
  logStep(`Waiting for server at ${url}`);
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(url, { cache: "no-store" });
      if (response.ok) {
        logStep("Server is ready");
        return;
      }
    } catch {
      // ignore connection errors while server starts
    }
    await delay(250);
  }
  throw new Error(`Server did not become ready within ${timeoutMs}ms`);
}

function parsePlaylistDurations(playlistText) {
  const durations = [];
  const lines = playlistText.split("\n");
  for (const line of lines) {
    if (line.startsWith("#EXTINF:")) {
      const value = line.slice("#EXTINF:".length).split(",")[0];
      const duration = Number.parseFloat(value);
      if (Number.isFinite(duration)) {
        durations.push(duration);
      }
    }
  }
  return durations;
}

async function startServer() {
  serverProcess = spawn("cargo", ["run", "--", INPUT_FILE], {
    cwd: PROJECT_ROOT,
    stdio: ["ignore", "pipe", "pipe"],
  });
  serverProcess.stdout.setEncoding("utf8");
  serverProcess.stderr.setEncoding("utf8");
  serverProcess.stdout.on("data", (chunk) => {
    diagnostics.serverStdout.push(chunk);
    console.log(`[hls-server] ${chunk.toString().trimEnd()}`);
  });
  serverProcess.stderr.on("data", (chunk) => {
    diagnostics.serverStderr.push(chunk);
    console.warn(`[hls-server] ${chunk.toString().trimEnd()}`);
  });
  await waitForServerReady(SERVER_URL);
}

async function stopServer() {
  if (!serverProcess) {
    return;
  }
  serverProcess.kill("SIGINT");
  await new Promise((resolve) => {
    serverProcess.once("exit", resolve);
    setTimeout(resolve, 3000);
  });
  serverProcess = undefined;
}

async function setupPlayer(page) {
  await page.goto(`${SERVER_ROOT}/`, { waitUntil: "domcontentloaded" });
  await page.evaluate(() => {
    document.open();
    document.write(`<!doctype html>
      <html>
        <head>
          <meta charset="utf-8" />
          <style>
            html, body { margin: 0; padding: 0; }
            video { width: 640px; height: 360px; background: #000; }
          </style>
        </head>
        <body>
          <video id="video" controls></video>
        </body>
      </html>`);
    document.close();
  });

  page.on("console", (msg) => {
    diagnostics.pageConsole.push({
      type: msg.type(),
      text: msg.text(),
    });
    console.log(`[page:${msg.type()}] ${msg.text()}`);
  });
  page.on("pageerror", (err) => {
    diagnostics.pageErrors.push(err.toString());
    console.error(`[page:error] ${err.toString()}`);
  });

  logStep("Injecting hls.js and wiring player");
  await page.addScriptTag({ path: HLS_SCRIPT });
  await page.evaluate((url) => {
    window.__hlsWarnings = [];
    window.__hlsErrors = [];
    window.__mediaErrors = [];
    window.__consoleWarnings = [];
    window.__consoleErrors = [];

    const video = document.getElementById("video");
    video.muted = true;
    video.autoplay = true;
    video.playsInline = true;

    window.addEventListener("error", (event) => {
      if (event?.message) {
        window.__consoleErrors.push(event.message);
      }
    });
    window.addEventListener("unhandledrejection", (event) => {
      if (event?.reason) {
        window.__consoleErrors.push(event.reason.toString());
      }
    });

    const logger = {
      debug: () => {},
      log: () => {},
      info: () => {},
      warn: (...args) => {
        window.__consoleWarnings.push(args.map(String).join(" "));
      },
      error: (...args) => {
        window.__consoleErrors.push(args.map(String).join(" "));
      },
    };

    const hls = new Hls({
      enableWorker: true,
      debug: true,
      logger,
    });

    window.__hlsInstance = hls;

    hls.on(Hls.Events.ERROR, (_event, data) => {
      const entry = {
        type: data.type,
        details: data.details,
        fatal: data.fatal,
      };
      if (data.fatal) {
        window.__hlsErrors.push(entry);
      } else {
        window.__hlsWarnings.push(entry);
      }
    });

    video.addEventListener("error", () => {
      window.__mediaErrors.push(video.error?.code ?? "unknown");
    });

    hls.loadSource(url);
    hls.attachMedia(video);
  }, SERVER_URL);
}

async function waitForPlaybackStart(page, timeoutMs = 20000) {
  logStep("Waiting for initial playback");
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const status = await page.evaluate(() => {
      const video = document.getElementById("video");
      return {
        isPlaying:
          video &&
          !video.paused &&
          video.readyState >= 3 &&
          video.currentTime > 0.2,
        hlsWarnings: window.__hlsWarnings ?? [],
        hlsErrors: window.__hlsErrors ?? [],
        mediaErrors: window.__mediaErrors ?? [],
        consoleWarnings: window.__consoleWarnings ?? [],
        consoleErrors: window.__consoleErrors ?? [],
      };
    });
    if (
      status.hlsWarnings.length ||
      status.hlsErrors.length ||
      status.mediaErrors.length ||
      status.consoleWarnings.length ||
      status.consoleErrors.length
    ) {
      throw new Error(
        `Playback failed early: ${JSON.stringify({
          hlsWarnings: status.hlsWarnings,
          hlsErrors: status.hlsErrors,
          mediaErrors: status.mediaErrors,
          consoleWarnings: status.consoleWarnings,
          consoleErrors: status.consoleErrors,
        })}`
      );
    }
    if (status.isPlaying) {
      return;
    }
    await delay(250);
  }
  throw new Error(`Playback did not start within ${timeoutMs}ms`);
}

async function waitForPlaybackSeconds(page, seconds, timeoutMs = 20000) {
  const startTime = await page.evaluate(() => {
    const video = document.getElementById("video");
    return video.currentTime;
  });
  logStep(`Waiting for playback to advance ${seconds}s`);
  const startedAt = Date.now();
  const target = startTime + seconds;
  while (Date.now() - startedAt < timeoutMs) {
    const status = await page.evaluate((targetTime) => {
      const video = document.getElementById("video");
      return {
        advanced: video && !video.paused && video.currentTime >= targetTime,
        hlsWarnings: window.__hlsWarnings ?? [],
        hlsErrors: window.__hlsErrors ?? [],
        mediaErrors: window.__mediaErrors ?? [],
        consoleWarnings: window.__consoleWarnings ?? [],
        consoleErrors: window.__consoleErrors ?? [],
      };
    }, target);
    if (
      status.hlsWarnings.length ||
      status.hlsErrors.length ||
      status.mediaErrors.length ||
      status.consoleWarnings.length ||
      status.consoleErrors.length
    ) {
      throw new Error(
        `Playback error while waiting: ${JSON.stringify({
          hlsWarnings: status.hlsWarnings,
          hlsErrors: status.hlsErrors,
          mediaErrors: status.mediaErrors,
          consoleWarnings: status.consoleWarnings,
          consoleErrors: status.consoleErrors,
        })}`
      );
    }
    if (status.advanced) {
      return;
    }
    await delay(250);
  }
  throw new Error(`Playback did not advance ${seconds}s within ${timeoutMs}ms`);
}

async function seekTo(page, timeSeconds) {
  logStep(`Seeking to ${timeSeconds.toFixed(2)}s`);
  await page.evaluate((seekTime) => {
    const video = document.getElementById("video");
    return new Promise((resolve, reject) => {
      const onSeeked = () => {
        video.removeEventListener("seeked", onSeeked);
        resolve();
      };
      const onError = () => {
        video.removeEventListener("seeked", onSeeked);
        reject(new Error("Media error during seek"));
      };
      video.addEventListener("seeked", onSeeked, { once: true });
      video.addEventListener("error", onError, { once: true });
      video.currentTime = seekTime;
      video.play().catch(() => {});
    });
  }, timeSeconds);
}

async function assertNoHlsIssues(page) {
  const { warnings, errors, mediaErrors, consoleWarnings, consoleErrors } =
    await page.evaluate(() => {
      return {
        warnings: window.__hlsWarnings,
        errors: window.__hlsErrors,
        mediaErrors: window.__mediaErrors,
        consoleWarnings: window.__consoleWarnings,
        consoleErrors: window.__consoleErrors,
      };
    });
  if (
    warnings.length ||
    errors.length ||
    mediaErrors.length ||
    consoleWarnings.length ||
    consoleErrors.length
  ) {
    console.error("[hls-test] HLS diagnostics", {
      warnings,
      errors,
      mediaErrors,
      consoleWarnings,
      consoleErrors,
    });
  }
  expect(mediaErrors).toEqual([]);
  expect(warnings).toEqual([]);
  expect(errors).toEqual([]);
  expect(consoleWarnings).toEqual([]);
  expect(consoleErrors).toEqual([]);
}

async function dumpDiagnostics(page) {
  let pageState = null;
  try {
    pageState = await page.evaluate(() => {
      const video = document.getElementById("video");
      return {
        currentTime: video?.currentTime ?? null,
        readyState: video?.readyState ?? null,
        paused: video?.paused ?? null,
        buffered: video?.buffered?.length ?? 0,
        bufferedRanges: video
          ? Array.from({ length: video.buffered.length }, (_, i) => [
              video.buffered.start(i),
              video.buffered.end(i),
            ])
          : [],
        hlsWarnings: window.__hlsWarnings ?? [],
        hlsErrors: window.__hlsErrors ?? [],
        mediaErrors: window.__mediaErrors ?? [],
        consoleWarnings: window.__consoleWarnings ?? [],
        consoleErrors: window.__consoleErrors ?? [],
      };
    });
  } catch (error) {
    pageState = { error: error?.toString?.() ?? "unknown" };
  }

  console.error("[hls-test] Page diagnostics", pageState);
  console.error("[hls-test] Collected console messages", diagnostics.pageConsole);
  console.error("[hls-test] Collected page errors", diagnostics.pageErrors);
}

beforeAll(async () => {
  await startServer();
});

afterAll(async () => {
  await stopServer();
});

test(
  "HLS playback covers basic streaming flows",
  async () => {
    const playlistText = await fetch(SERVER_URL).then((res) => res.text());
    const durations = parsePlaylistDurations(playlistText);
    const totalDuration = durations.reduce((sum, value) => sum + value, 0);
    expect(durations.length).toBeGreaterThan(1);
    expect(totalDuration).toBeGreaterThan(70);

    const browserExecutable = findBrowserExecutable();
    const browser = await puppeteer.launch({
      headless: "new",
      args: ["--autoplay-policy=no-user-gesture-required"],
      ...(browserExecutable ? { executablePath: browserExecutable } : {}),
    });
    const page = await browser.newPage();
    try {
      await setupPlayer(page);
      await waitForPlaybackStart(page);
      await waitForPlaybackSeconds(page, 5);
      await assertNoHlsIssues(page);

      await seekTo(page, 60);
      await waitForPlaybackSeconds(page, 5);
      await assertNoHlsIssues(page);

      let segmentIndex = durations.findIndex((duration) => duration > 5);
      if (segmentIndex === -1 || segmentIndex === durations.length - 1) {
        segmentIndex = 0;
      }
      const boundaryTime = durations
        .slice(0, segmentIndex + 1)
        .reduce((sum, value) => sum + value, 0);
      const seekTime = Math.max(boundaryTime - 5, 0);
      await seekTo(page, seekTime);
      await waitForPlaybackSeconds(page, 10);
      await assertNoHlsIssues(page);

      const playbackInfo = await page.evaluate(() => {
        const video = document.getElementById("video");
        const hls = window.__hlsInstance;
        return {
          videoWidth: video.videoWidth,
          videoHeight: video.videoHeight,
          audioTracks: hls?.audioTracks?.length ?? 0,
          levels: hls?.levels?.length ?? 0,
          audioDecoded: video.webkitAudioDecodedByteCount ?? null,
          videoDecoded: video.webkitVideoDecodedByteCount ?? null,
          currentTime: video.currentTime,
        };
      });

      expect(playbackInfo.videoWidth).toBeGreaterThan(0);
      expect(playbackInfo.videoHeight).toBeGreaterThan(0);
      expect(playbackInfo.levels).toBeGreaterThan(0);
      if (typeof playbackInfo.audioDecoded === "number") {
        expect(playbackInfo.audioDecoded).toBeGreaterThan(0);
      } else {
        expect(playbackInfo.audioTracks).toBeGreaterThan(0);
      }
      if (typeof playbackInfo.videoDecoded === "number") {
        expect(playbackInfo.videoDecoded).toBeGreaterThan(0);
      }
      expect(playbackInfo.currentTime).toBeGreaterThan(0);
      logStep("Playback validations completed");
    } catch (error) {
      await dumpDiagnostics(page);
      throw error;
    } finally {
      await page.close();
      await browser.close();
    }
  },
  120000
);
