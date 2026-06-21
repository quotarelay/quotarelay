import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

const controlPlaneRoot = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(controlPlaneRoot, "..", "..");
const backendHost = "127.0.0.1";
const backendPort = 3030;
const frontendHost = "127.0.0.1";
const frontendPort = 4174;
const backendAddr = `${backendHost}:${backendPort}`;
const truthUrl = `http://${backendAddr}/truth`;
const args = process.argv.slice(2);
const mode = args.includes("--headless")
  ? "headless"
  : args.includes("--dev")
    ? "dev"
    : "serve";
const backendBin = join(
  repoRoot,
  "target",
  "debug",
  process.platform === "win32" ? "mcp-server.exe" : "mcp-server",
);
const viteBin = join(
  controlPlaneRoot,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "vite.cmd" : "vite",
);

let backendProcess = null;
let frontendProcess = null;
let shuttingDown = false;

async function canReadTruth() {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 1200);

  try {
    const response = await fetch(truthUrl, { signal: controller.signal });
    return response.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
}

async function waitForTruth() {
  const started = Date.now();
  while (Date.now() - started < 120000) {
    if (await canReadTruth()) {
      return true;
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 1000));
  }
  return false;
}

function runCommand(file, args, cwd) {
  return new Promise((resolveCommand, rejectCommand) => {
    const child = spawn(file, args, {
      cwd,
      env: process.env,
      stdio: "inherit",
    });

    child.on("error", rejectCommand);
    child.on("exit", (code) => {
      if (code === 0) {
        resolveCommand();
        return;
      }
      rejectCommand(new Error(`${file} exited with code ${code ?? 1}.`));
    });
  });
}

async function buildBackend() {
  console.log("Building backend binary");
  await runCommand("cargo", ["build", "-p", "mcp-server"], repoRoot);
  if (!existsSync(backendBin)) {
    throw new Error(`Backend binary was not found at ${backendBin}`);
  }
}

async function buildFrontend() {
  if (!existsSync(viteBin)) {
    throw new Error("Vite is not installed. Run npm --prefix web/controlplane ci.");
  }

  console.log("Building dashboard");
  await runCommand(viteBin, ["build"], controlPlaneRoot);
}

function startBackend() {
  backendProcess = spawn(backendBin, ["--http", backendAddr], {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
  });

  backendProcess.on("error", (error) => {
    console.error(error.message);
    stopChildren(1);
  });
  backendProcess.on("exit", (code) => {
    if (shuttingDown) {
      return;
    }
    console.error(`Backend exited with code ${code ?? 1}.`);
    stopChildren(code ?? 1);
  });
}

function startFrontend() {
  if (!existsSync(viteBin)) {
    console.error("Vite is not installed. Run npm --prefix web/controlplane ci.");
    stopChildren(1);
    return;
  }

  const viteArgs =
    mode === "dev"
      ? ["--host", frontendHost, "--port", `${frontendPort}`]
      : ["preview", "--host", frontendHost, "--port", `${frontendPort}`];
  frontendProcess = spawn(viteBin, viteArgs, {
    cwd: controlPlaneRoot,
    env: process.env,
    stdio: "inherit",
  });

  frontendProcess.on("error", (error) => {
    console.error(error.message);
    stopChildren(1);
  });
  frontendProcess.on("exit", (code) => {
    if (shuttingDown) {
      return;
    }
    stopChildren(code ?? 0);
  });
}

function stopChildren(code = 0) {
  if (shuttingDown) {
    return;
  }
  shuttingDown = true;

  if (frontendProcess && !frontendProcess.killed) {
    frontendProcess.kill();
  }
  if (backendProcess && !backendProcess.killed) {
    backendProcess.kill();
  }

  process.exit(code);
}

process.on("SIGINT", () => stopChildren(0));
process.on("SIGTERM", () => stopChildren(0));

async function main() {
  if (args.includes("--help")) {
    console.log("Usage: node dev-with-backend.mjs [--headless|--dev]");
    console.log("Default: build and serve the dashboard from production assets.");
    console.log("--headless: start only the loopback HTTP backend.");
    console.log("--dev: start backend plus the frontend dev server.");
    return;
  }

  if (mode === "serve") {
    await buildFrontend();
  }

  if (await canReadTruth()) {
    console.log(`Backend already available at ${truthUrl}`);
  } else {
    await buildBackend();
    console.log(`Starting backend at ${truthUrl}`);
    startBackend();
    if (!(await waitForTruth())) {
      console.error("Backend did not become ready within 120 seconds.");
      stopChildren(1);
    }
  }

  if (mode === "headless") {
    console.log(`Headless backend ready at ${truthUrl}`);
    return;
  }

  console.log(`Starting dashboard at http://${frontendHost}:${frontendPort}/`);
  startFrontend();
}

await main().catch((error) => {
  console.error(error.message);
  stopChildren(1);
});
