import path from "node:path";

export async function attachHlsPlayer({ page, hlsScriptPath, playlistUrl }) {
  await page.setContent(`<!doctype html>
    <html>
      <head><meta charset="utf-8"></head>
      <body>
        <video id="video" muted playsinline autoplay></video>
      </body>
    </html>`);

  await page.addScriptTag({ path: hlsScriptPath });

  await page.evaluate((playlist) => {
    window.__hlsTest = { warnings: [], errors: [] };

    const video = document.getElementById("video");
    const logger = {
      log() {},
      debug() {},
      info() {},
      warn: (...args) => window.__hlsTest.warnings.push(args.join(" ")),
      error: (...args) => window.__hlsTest.errors.push(args.join(" ")),
    };

    if (!window.Hls.isSupported()) {
      throw new Error("hls.js is not supported in this browser");
    }

    const hls = new window.Hls({
      debug: true,
      logger,
    });

    window.__hls = hls;

    hls.on(window.Hls.Events.ERROR, (_evt, data) => {
      window.__hlsTest.errors.push({
        type: data?.type ?? null,
        details: data?.details ?? null,
        fatal: Boolean(data?.fatal),
        responseCode: data?.response?.code ?? null,
      });
    });

    hls.loadSource(playlist);
    hls.attachMedia(video);

    return new Promise((resolve, reject) => {
      hls.on(window.Hls.Events.MANIFEST_PARSED, () => {
        video
          .play()
          .then(() => resolve())
          .catch((err) => reject(err));
      });
    });
  }, playlistUrl);
}

export async function waitForPlayback(page, minAdvanceSeconds = 0.5) {
  await page.waitForFunction(() => {
    const video = document.querySelector("video");
    return video && !video.paused && video.readyState >= 2;
  });

  const startTime = await page.evaluate(() => {
    const video = document.querySelector("video");
    return video ? video.currentTime : 0;
  });

  await page.waitForFunction(
    ([start, minAdvance]) => {
      const video = document.querySelector("video");
      return (
        video &&
        !video.paused &&
        video.currentTime >= start + minAdvance
      );
    },
    [startTime, minAdvanceSeconds],
    { timeout: 30_000 }
  );
}

export async function assertNoHlsWarnings(page, label) {
  const { warnings, errors } = await page.evaluate(() => window.__hlsTest);
  if (errors.length > 0) {
    throw new Error(
      `hls.js error events after ${label}: ${JSON.stringify(errors)}`
    );
  }
  if (warnings.length > 0) {
    throw new Error(
      `hls.js logger warnings after ${label}: ${JSON.stringify(warnings)}`
    );
  }
}

export function assertNoConsoleWarnings(consoleMessages, label) {
  const hlsWarningPattern =
    /(hls|transmux|remux|passthrough-remuxer|demux|mux)/i;
  const hlsConsoleWarnings = consoleMessages.filter((message) =>
    hlsWarningPattern.test(message.text)
  );

  if (hlsConsoleWarnings.length > 0) {
    throw new Error(
      `hls.js console warnings after ${label}: ${JSON.stringify(
        hlsConsoleWarnings
      )}`
    );
  }
}

export function getHlsScriptPath(testDir) {
  return path.resolve(testDir, "node_modules/hls.js/dist/hls.min.js");
}
