import { execSync } from "node:child_process";
import process from "node:process";

const port = process.argv[2];
if (!port) {
  console.error("Usage: node free-port.mjs <port>");
  process.exit(1);
}

const isWin = process.platform === "win32";

function getListenerPid(port) {
  try {
    if (isWin) {
      const output = execSync(`netstat -ano | findstr :${port}`, {
        encoding: "utf8",
        stdio: ["pipe", "pipe", "ignore"],
      });
      const lines = output.split("\n").map((line) => line.trim()).filter(Boolean);
      for (const line of lines) {
        const parts = line.split(/\s+/);
        // Format: Proto  Local Address          Foreign Address        State           PID
        const proto = parts[0];
        const local = parts[1];
        const state = parts[3];
        const pid = parts[4];
        if (
          (proto === "TCP" || proto === "TCP6") &&
          local?.endsWith(":" + port) &&
          state === "LISTENING" &&
          pid &&
          /^\d+$/.test(pid)
        ) {
          return pid;
        }
      }
    } else {
      const output = execSync(`lsof -ti :${port} -s TCP:LISTEN`, {
        encoding: "utf8",
        stdio: ["pipe", "pipe", "ignore"],
      });
      const pid = output.trim().split("\n")[0];
      if (pid && /^\d+$/.test(pid)) {
        return pid;
      }
    }
  } catch {
    // no listener found
  }
  return null;
}

function getProcessName(pid) {
  try {
    if (isWin) {
      const output = execSync(`tasklist /FI "PID eq ${pid}" /FO CSV /NH`, {
        encoding: "utf8",
        stdio: ["pipe", "pipe", "ignore"],
      });
      const match = output.match(/^"([^"]+)"/);
      return match?.[1] ?? null;
    } else {
      const output = execSync(`ps -p ${pid} -o comm=`, {
        encoding: "utf8",
        stdio: ["pipe", "pipe", "ignore"],
      });
      return output.trim() || null;
    }
  } catch {
    return null;
  }
}

const pid = getListenerPid(port);
if (!pid) {
  process.exit(0);
}

const name = getProcessName(pid);
if (name) {
  const allowed = ["node.exe", "node", "vite"];
  const base = name.replace(/\.exe$/i, "");
  if (!allowed.includes(name) && !allowed.includes(base)) {
    console.error(
      `Port ${port} is held by an unexpected process (${name}, PID ${pid}). Please free it manually.`
    );
    process.exit(1);
  }
}

try {
  if (isWin) {
    execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
  } else {
    execSync(`kill -9 ${pid}`, { stdio: "ignore" });
  }
  console.log(`Freed port ${port} (killed ${name ?? "process"} ${pid})`);
} catch {
  console.error(`Failed to free port ${port} (PID ${pid}).`);
  process.exit(1);
}
