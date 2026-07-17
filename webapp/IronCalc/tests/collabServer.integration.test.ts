import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { afterAll, beforeAll, expect, test } from "vitest";
import { CollabProvider } from "../src/collab/CollabProvider";

// Integration check: two providers through the real relay server over real
// websockets (node's global WebSocket). Skipped when the server binary is
// not built (`cargo build -p ironcalc_collab_server`).

const SERVER_BINARY = "../../target/debug/ironcalc_collab_server";
const hasServer = existsSync(SERVER_BINARY);

const PORT = 19837;
let server: ReturnType<typeof spawn>;

beforeAll(async () => {
  if (!hasServer) {
    return;
  }
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
  server = spawn(SERVER_BINARY, [`127.0.0.1:${PORT}`], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  await new Promise<void>((resolve, reject) => {
    server.stderr?.on("data", (chunk: Buffer) => {
      if (chunk.toString().includes("collab relay")) {
        resolve();
      }
    });
    server.on("error", reject);
    setTimeout(() => reject(new Error("server did not start")), 5000);
  });
});

afterAll(() => {
  server?.kill();
});

function waitFor(check: () => boolean, ms = 5000): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const timer = setInterval(() => {
      if (check()) {
        clearInterval(timer);
        resolve();
      } else if (Date.now() - start > ms) {
        clearInterval(timer);
        reject(new Error("condition not met in time"));
      }
    }, 25);
  });
}

test.runIf(hasServer)(
  "two providers converge through the real relay server",
  async () => {
    const a = new Model("workbook", "en", "UTC", "en");
    const b = new Model("workbook", "en", "UTC", "en");
    const url = `ws://127.0.0.1:${PORT}/it-room`;
    const pa = new CollabProvider(a, url, { clientId: 1, flushIntervalMs: 50 });
    const pb = new CollabProvider(b, url, { clientId: 2, flushIntervalMs: 50 });
    try {
      pa.connect();
      pb.connect();
      await waitFor(
        () => pa.status === "connected" && pb.status === "connected",
      );

      a.setUserInput(0, 1, 1, "21");
      await waitFor(() => b.getFormattedCellValue(0, 1, 1) === "21");

      b.setUserInput(0, 1, 2, "=A1*2");
      await waitFor(() => a.getFormattedCellValue(0, 1, 2) === "42");

      pa.setPresence({ name: "ana" });
      await waitFor(() =>
        pb.presence().some((p) => p.clientId === 1 && p.state.includes("ana")),
      );

      // A late joiner starts blank and receives the workbook.
      const c = new Model("workbook", "en", "UTC", "en");
      const pc = new CollabProvider(c, url, {
        clientId: 3,
        flushIntervalMs: 50,
      });
      try {
        pc.connect();
        await waitFor(() => c.getFormattedCellValue(0, 1, 2) === "42");
        expect(c.getFormattedCellValue(0, 1, 1)).toBe("21");
        await waitFor(() => pc.presence().some((p) => p.clientId === 1));
      } finally {
        pc.destroy();
      }
      await waitFor(() => !pb.presence().some((p) => p.clientId === 3), 2000);
    } finally {
      pa.destroy();
      pb.destroy();
    }
  },
  20000,
);
