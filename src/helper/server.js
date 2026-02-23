const http = require("node:http");
const crypto = require("node:crypto");
const { URL } = require("node:url");

const CLIP_TYPES = new Set(["full_page", "selection"]);

function json(res, statusCode, payload) {
  const body = JSON.stringify(payload);
  res.writeHead(statusCode, {
    "Content-Type": "application/json; charset=utf-8",
    "Content-Length": Buffer.byteLength(body)
  });
  res.end(body);
}

function setCorsHeaders(res) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET,POST,DELETE,OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
}

function validateClipPayload(payload) {
  if (!payload || typeof payload !== "object") {
    return "Invalid payload.";
  }

  if (!CLIP_TYPES.has(payload.type)) {
    return "Invalid or missing clip type.";
  }

  const requiredStringFields = ["title", "url", "contentMarkdown", "createdAt"];
  for (const field of requiredStringFields) {
    if (typeof payload[field] !== "string" || payload[field].trim() === "") {
      return `Missing or invalid field: ${field}.`;
    }
  }

  if (Number.isNaN(Date.parse(payload.createdAt))) {
    return "Invalid createdAt. Must be ISO-8601 string.";
  }

  return null;
}

function readJsonBody(req, maxBodyBytes) {
  return new Promise((resolve, reject) => {
    let total = 0;
    const chunks = [];

    req.on("data", (chunk) => {
      total += chunk.length;
      if (total > maxBodyBytes) {
        const error = new Error("Payload too large.");
        error.statusCode = 413;
        reject(error);
        req.destroy();
        return;
      }

      chunks.push(chunk);
    });

    req.on("end", () => {
      const raw = Buffer.concat(chunks).toString("utf8").trim();
      if (!raw) {
        resolve({});
        return;
      }

      try {
        resolve(JSON.parse(raw));
      } catch {
        const error = new Error("Invalid JSON body.");
        error.statusCode = 400;
        reject(error);
      }
    });

    req.on("error", (error) => {
      reject(error);
    });
  });
}

function createStore(ttlMs) {
  const clips = new Map();

  function cleanupExpired() {
    const now = Date.now();
    for (const [clipId, record] of clips.entries()) {
      if (record.expiresAtMs <= now) {
        clips.delete(clipId);
      }
    }
  }

  function put(payload) {
    const clipId = crypto.randomUUID();
    const expiresAtMs = Date.now() + ttlMs;
    clips.set(clipId, { payload, expiresAtMs });
    return {
      clipId,
      expiresAt: new Date(expiresAtMs).toISOString()
    };
  }

  function get(clipId) {
    const record = clips.get(clipId);
    if (!record) {
      return null;
    }

    if (record.expiresAtMs <= Date.now()) {
      clips.delete(clipId);
      return null;
    }

    return {
      clipId,
      payload: record.payload,
      expiresAt: new Date(record.expiresAtMs).toISOString()
    };
  }

  function remove(clipId) {
    return clips.delete(clipId);
  }

  function size() {
    cleanupExpired();
    return clips.size;
  }

  return {
    cleanupExpired,
    get,
    put,
    remove,
    size
  };
}

function createServer(options = {}) {
  const host = options.host || "127.0.0.1";
  const port = Number.isInteger(options.port) ? options.port : 27124;
  const ttlMs = Number.isInteger(options.ttlMs) ? options.ttlMs : 15 * 60 * 1000;
  const maxBodyBytes = Number.isInteger(options.maxBodyBytes)
    ? options.maxBodyBytes
    : 5 * 1024 * 1024;

  const store = createStore(ttlMs);

  const server = http.createServer(async (req, res) => {
    setCorsHeaders(res);

    if (!req.url || !req.method) {
      json(res, 400, { error: "Invalid request." });
      return;
    }

    if (req.method === "OPTIONS") {
      res.writeHead(204);
      res.end();
      return;
    }

    let parsedUrl;
    try {
      parsedUrl = new URL(req.url, "http://localhost");
    } catch {
      json(res, 400, { error: "Invalid URL." });
      return;
    }

    if (req.method === "GET" && parsedUrl.pathname === "/health") {
      json(res, 200, { ok: true, clipsInMemory: store.size() });
      return;
    }

    if (req.method === "POST" && parsedUrl.pathname === "/clips") {
      try {
        const payload = await readJsonBody(req, maxBodyBytes);
        const validationError = validateClipPayload(payload);

        if (validationError) {
          json(res, 400, { error: validationError });
          return;
        }

        const result = store.put(payload);
        json(res, 201, result);
      } catch (error) {
        const statusCode = Number.isInteger(error.statusCode) ? error.statusCode : 500;
        json(res, statusCode, { error: error.message || "Failed to process clip." });
      }

      return;
    }

    const clipMatch = parsedUrl.pathname.match(/^\/clips\/([0-9a-fA-F-]+)$/);
    if (clipMatch && req.method === "GET") {
      const clipId = clipMatch[1];
      const record = store.get(clipId);
      if (!record) {
        json(res, 404, { error: "Clip not found or expired." });
        return;
      }

      json(res, 200, record);
      return;
    }

    if (clipMatch && req.method === "DELETE") {
      const clipId = clipMatch[1];
      const removed = store.remove(clipId);
      if (!removed) {
        json(res, 404, { error: "Clip not found." });
        return;
      }

      res.writeHead(204);
      res.end();
      return;
    }

    json(res, 404, { error: "Route not found." });
  });

  const cleanupTimer = setInterval(() => {
    store.cleanupExpired();
  }, Math.min(60 * 1000, Math.max(5 * 1000, Math.floor(ttlMs / 2))));

  if (typeof cleanupTimer.unref === "function") {
    cleanupTimer.unref();
  }

  function start() {
    return new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(port, host, () => {
        server.off("error", reject);
        resolve();
      });
    });
  }

  function stop() {
    return new Promise((resolve, reject) => {
      clearInterval(cleanupTimer);
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve();
      });
    });
  }

  function baseUrl() {
    const address = server.address();
    if (!address || typeof address === "string") {
      return null;
    }

    const resolvedHost = address.address === "::" ? "127.0.0.1" : address.address;
    return `http://${resolvedHost}:${address.port}`;
  }

  return {
    baseUrl,
    start,
    stop
  };
}

if (require.main === module) {
  const app = createServer({
    host: process.env.SNORGN_HELPER_HOST || "127.0.0.1",
    port: process.env.SNORGN_HELPER_PORT ? Number(process.env.SNORGN_HELPER_PORT) : 27124,
    ttlMs: process.env.SNORGN_HELPER_TTL_MS ? Number(process.env.SNORGN_HELPER_TTL_MS) : undefined,
    maxBodyBytes: process.env.SNORGN_HELPER_MAX_BODY_BYTES
      ? Number(process.env.SNORGN_HELPER_MAX_BODY_BYTES)
      : undefined
  });

  app.start()
    .then(() => {
      const listeningUrl = app.baseUrl();
      console.log(`Snorgnote helper API is running on ${listeningUrl}`);
    })
    .catch((error) => {
      console.error("Failed to start helper API:", error);
      process.exitCode = 1;
    });

  const shutdown = () => {
    app.stop()
      .catch(() => {})
      .finally(() => {
        process.exit(0);
      });
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

module.exports = {
  createServer,
  validateClipPayload
};
