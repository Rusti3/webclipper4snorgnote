const test = require("node:test");
const assert = require("node:assert/strict");

const { createServer } = require("../src/helper/server");

async function startTestServer(overrides = {}) {
  const app = createServer({
    host: "127.0.0.1",
    port: 0,
    ...overrides
  });

  await app.start();
  return app;
}

test("POST /clips accepts valid payload and returns clipId", async () => {
  const app = await startTestServer();
  const baseUrl = app.baseUrl();

  try {
    const payload = {
      type: "full_page",
      title: "Example",
      url: "https://example.com/page",
      contentMarkdown: "# Heading",
      createdAt: new Date().toISOString()
    };

    const response = await fetch(`${baseUrl}/clips`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload)
    });

    assert.equal(response.status, 201);
    const body = await response.json();

    assert.ok(body.clipId);
    assert.ok(body.expiresAt);
  } finally {
    await app.stop();
  }
});

test("GET /clips/:id returns previously stored payload", async () => {
  const app = await startTestServer();
  const baseUrl = app.baseUrl();

  try {
    const payload = {
      type: "selection",
      title: "Selection clip",
      url: "https://example.com/article",
      contentMarkdown: "selected text",
      createdAt: new Date().toISOString()
    };

    const createResponse = await fetch(`${baseUrl}/clips`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload)
    });
    const createBody = await createResponse.json();

    const getResponse = await fetch(`${baseUrl}/clips/${createBody.clipId}`);
    assert.equal(getResponse.status, 200);

    const getBody = await getResponse.json();
    assert.equal(getBody.payload.title, payload.title);
    assert.equal(getBody.payload.contentMarkdown, payload.contentMarkdown);
    assert.equal(getBody.payload.type, payload.type);
  } finally {
    await app.stop();
  }
});

test("POST /clips rejects invalid payload", async () => {
  const app = await startTestServer();
  const baseUrl = app.baseUrl();

  try {
    const invalidPayload = {
      type: "full_page",
      title: "",
      url: "https://example.com/page",
      contentMarkdown: "",
      createdAt: "not-a-date"
    };

    const response = await fetch(`${baseUrl}/clips`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(invalidPayload)
    });

    assert.equal(response.status, 400);
    const body = await response.json();
    assert.match(body.error, /missing|invalid/i);
  } finally {
    await app.stop();
  }
});

test("clip expires after TTL", async () => {
  const app = await startTestServer({ ttlMs: 60 });
  const baseUrl = app.baseUrl();

  try {
    const payload = {
      type: "full_page",
      title: "TTL test",
      url: "https://example.com/page",
      contentMarkdown: "ttl",
      createdAt: new Date().toISOString()
    };

    const createResponse = await fetch(`${baseUrl}/clips`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload)
    });
    const createBody = await createResponse.json();

    await new Promise((resolve) => setTimeout(resolve, 90));

    const getResponse = await fetch(`${baseUrl}/clips/${createBody.clipId}`);
    assert.equal(getResponse.status, 404);
  } finally {
    await app.stop();
  }
});
