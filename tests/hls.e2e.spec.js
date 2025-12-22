import { afterAll, beforeAll, test } from "bun:test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "./helpers/browser.js";
import { startServer, stopServer } from "./helpers/server.js";
import { parsePlaylistSegments } from "./helpers/playlist.js";
import {
  attachHlsPlayer,
  waitForPlayback,
  assertNoHlsWarnings,
  assertNoConsoleWarnings,
  getHlsScriptPath,
} from "./helpers/hls-player.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const serverUrl = "http://127.0.0.1:3101";
const playlistUrl = `${serverUrl}/hls/test/index.m3u8`;
const seekTargetSeconds = 300;

let serverProcess;
let browser;
let segments = [];

beforeAll(
  async () => {
    serverProcess = await startServer({
      cwd: repoRoot,
      bindAddr: "127.0.0.1:3101",
    });

    const playlistRes = await fetch(playlistUrl, { cache: "no-store" });
    if (!playlistRes.ok) {
      throw new Error(`Playlist fetch failed with ${playlistRes.status}`);
    }
    const playlistText = await playlistRes.text();
    segments = parsePlaylistSegments(playlistText);
    console.log(`[hls-e2e] Parsed ${segments.length} segments from playlist.`);
    if (segments.length < 3) {
      throw new Error("Not enough segments to run seek tests");
    }

    browser = await launchBrowser();
    console.log("[hls-e2e] Browser launched.");
  },
  { timeout: 120_000 }
);

afterAll(async () => {
  if (browser) {
    await browser.close();
  }
  await stopServer(serverProcess);
});

test(
  "plays and seeks without hls.js warnings",
  async () => {
    const fiveMinuteIndex = segments.findIndex(
      (segment) =>
        segment.start <= seekTargetSeconds &&
        segment.start + segment.duration > seekTargetSeconds
    );

    if (fiveMinuteIndex < 1) {
      throw new Error("Could not locate segment before 5 minute mark");
    }

    const priorSegment = segments[fiveMinuteIndex - 1];
    const priorMidpoint = priorSegment.start + priorSegment.duration / 2;
    const priorCrossTime = priorSegment.start + priorSegment.duration + 1;

    console.log(
      `[hls-e2e] Seeking to 5:00 in segment ${segments[fiveMinuteIndex].id}, ` +
        `then mid-segment ${priorSegment.id} at ${priorMidpoint.toFixed(2)}s.`
    );

    const page = await browser.newPage();
    const consoleMessages = [];
    page.on("console", (message) => {
      const type = message.type();
      if (type === "warning" || type === "error") {
        consoleMessages.push({ type, text: message.text() });
      }
    });
    try {
      console.log("[hls-e2e] Loading hls.js test page...");
      await attachHlsPlayer({
        page,
        hlsScriptPath: getHlsScriptPath(__dirname),
        playlistUrl,
      });

      console.log("[hls-e2e] Waiting for initial playback...");
      await waitForPlayback(page, 1.0);
      await assertNoHlsWarnings(page, "initial playback");
      assertNoConsoleWarnings(consoleMessages, "initial playback");

      console.log("[hls-e2e] Seeking to 5:00...");
      await page.evaluate((timeSeconds) => {
        const video = document.querySelector("video");
        video.currentTime = timeSeconds;
        return video.play();
      }, seekTargetSeconds);

      await waitForPlayback(page, 1.0);
      await assertNoHlsWarnings(page, "seek to 5 minutes");
      assertNoConsoleWarnings(consoleMessages, "seek to 5 minutes");

      console.log("[hls-e2e] Seeking into prior segment and crossing boundary...");
      await page.evaluate((timeSeconds) => {
        const video = document.querySelector("video");
        video.currentTime = timeSeconds;
        return video.play();
      }, priorMidpoint);

      await page.waitForFunction(
        (targetTime) => {
          const video = document.querySelector("video");
          return video && !video.paused && video.currentTime >= targetTime;
        },
        priorCrossTime,
        { timeout: 45_000 }
      );

      await assertNoHlsWarnings(page, "seek across segment boundary");
      assertNoConsoleWarnings(consoleMessages, "seek across segment boundary");
      console.log("[hls-e2e] Playback checks passed.");
    } finally {
      await page.close();
    }
  },
  120_000
);
