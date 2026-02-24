const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const BACKGROUND_PATH = path.join(__dirname, "..", "src", "extension", "background.js");

function loadBackground({ sendMessageResult, sendMessageError } = {}) {
  const calls = {
    sendMessage: [],
    create: [],
  };

  const listeners = {
    onInstalled: null,
    onStartup: null,
    onContextMenuClicked: null,
    onRuntimeMessage: null,
  };

  const chrome = {
    storage: {
      local: {
        async get() {
          return {};
        },
        async set() {},
      },
    },
    action: {
      async setBadgeBackgroundColor() {},
      async setBadgeText() {},
    },
    contextMenus: {
      async removeAll() {},
      create() {},
      onClicked: {
        addListener(handler) {
          listeners.onContextMenuClicked = handler;
        },
      },
    },
    tabs: {
      async query() {
        return [{ id: 7, url: "https://example.com", title: "Example" }];
      },
      async sendMessage(tabId, payload) {
        calls.sendMessage.push({ tabId, payload });
        if (sendMessageError) {
          throw sendMessageError;
        }
        if (typeof sendMessageResult !== "undefined") {
          return sendMessageResult;
        }
        return { ok: true };
      },
      async create(payload) {
        calls.create.push(payload);
        return { id: 99, ...payload };
      },
    },
    runtime: {
      onInstalled: {
        addListener(handler) {
          listeners.onInstalled = handler;
        },
      },
      onStartup: {
        addListener(handler) {
          listeners.onStartup = handler;
        },
      },
      onMessage: {
        addListener(handler) {
          listeners.onRuntimeMessage = handler;
        },
      },
    },
  };

  const context = {
    console,
    chrome,
    importScripts() {},
    setTimeout() {
      return 0;
    },
    clearTimeout() {},
    crypto: {
      randomUUID() {
        return "test-uuid";
      },
    },
    self: {
      SnorgPayload: {
        clipPayloadToDeepLink() {
          return {
            deepLink: "snorgnote://new?data=test",
            clipped: false,
            originalLength: 10,
            finalLength: 10,
          };
        },
      },
    },
  };

  vm.createContext(context);
  const source = fs.readFileSync(BACKGROUND_PATH, "utf8");
  const testExports = "\n;globalThis.__test__ = { openAppWithPayload };";
  vm.runInContext(`${source}${testExports}`, context, { filename: "background.js" });

  return {
    api: context.__test__,
    calls,
    listeners,
  };
}

test("openAppWithPayload sends deep-link to current tab via content script", async () => {
  const { api, calls } = loadBackground({ sendMessageResult: { ok: true } });

  const encoded = await api.openAppWithPayload(7, {
    type: "selection",
    title: "Hello",
    url: "https://example.com",
    contentMarkdown: "text",
    createdAt: "2026-02-24T00:00:00.000Z",
    source: "web-clipper",
  });

  assert.equal(encoded.deepLink, "snorgnote://new?data=test");
  assert.equal(calls.sendMessage.length, 1);
  assert.equal(calls.sendMessage[0].tabId, 7);
  assert.equal(calls.sendMessage[0].payload.type, "open_deeplink");
  assert.equal(calls.sendMessage[0].payload.deepLink, "snorgnote://new?data=test");
  assert.equal(calls.create.length, 0);
});

test("openAppWithPayload surfaces content-script launch errors", async () => {
  const { api } = loadBackground({
    sendMessageResult: { ok: false, error: "Blocked by page policy" },
  });

  await assert.rejects(
    () =>
      api.openAppWithPayload(7, {
        type: "selection",
        title: "Hello",
        url: "https://example.com",
        contentMarkdown: "text",
        createdAt: "2026-02-24T00:00:00.000Z",
        source: "web-clipper",
      }),
    /Blocked by page policy/,
  );
});
