import readline from "node:readline";
import { spawn } from "node:child_process";

function waitForServerLog(server, timeoutMs = 45_000) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error("Server did not log startup message in time"));
    }, timeoutMs);

    const onLine = (line) => {
      if (line.includes("Server starting on")) {
        cleanup();
        resolve();
      }
    };

    const onExit = (code) => {
      cleanup();
      reject(new Error(`Server exited before ready (code ${code ?? "null"})`));
    };

    const stdout = readline.createInterface({ input: server.stdout });
    const stderr = readline.createInterface({ input: server.stderr });

    const cleanup = () => {
      clearTimeout(timeout);
      stdout.off("line", onLine);
      stderr.off("line", onLine);
      stdout.close();
      stderr.close();
      server.off("exit", onExit);
    };

    stdout.on("line", onLine);
    stderr.on("line", onLine);
    server.on("exit", onExit);
  });
}

export async function startServer({ cwd, bindAddr }) {
  const serverProcess = spawn("cargo", ["run"], {
    cwd,
    env: {
      ...process.env,
      RUST_LOG: process.env.RUST_LOG ?? "info",
      LYRA_BIND_ADDR: bindAddr,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });

  console.log("[hls-e2e] Waiting for server startup log...");
  await waitForServerLog(serverProcess);
  console.log("[hls-e2e] Server ready.");

  return serverProcess;
}

export async function stopServer(serverProcess) {
  if (!serverProcess) return;
  serverProcess.kill("SIGINT");
  await Promise.race([
    new Promise((resolve) => serverProcess.on("exit", resolve)),
    new Promise((resolve) => setTimeout(resolve, 5_000)),
  ]);
}
