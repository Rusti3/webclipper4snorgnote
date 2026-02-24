const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const BACKGROUND_PATH = path.join(__dirname, "..", "src", "extension", "background.js");

function loadBackground({ sendMessageResult, sendMessageError, updateError } = {}) {
  const calls = {
    sendMessage: [],
    create: [],
    getURL: [],
    update: [],
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
        if (payload && payload.type === "extract_page") {
          return {
            title: "Example title",
            url: "https://example.com/article",
            contentMarkdown: "Article body",
          };
        }
        if (payload && payload.type === "extract_selection") {
          return {
            selectionText: "Selected from content script",
          };
        }
        return { ok: true };
      },
      async create(payload) {
        calls.create.push(payload);
        return { id: 99, ...payload };
      },
      async update(tabId, payload) {
        calls.update.push({ tabId, payload });
        if (updateError) {
          throw updateError;
        }
        return { id: tabId, ...payload };
      },
    },
    runtime: {
      getURL(value) {
        calls.getURL.push(value);
        return `chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/${value}`;
      },
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
    URLSearchParams,
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
  const testExports = "\n;globalThis.__test__ = { buildLauncherUrl, openLauncherTab };";
  vm.runInContext(`${source}${testExports}`, context, { filename: "background.js" });

  return {
    api: context.__test__,
    calls,
    listeners,
  };
}

async function sendRuntimeMessage(listeners, message) {
  return new Promise((resolve, reject) => {
    if (typeof listeners.onRuntimeMessage !== "function") {
      reject(new Error("runtime listener is missing"));
      return;
    }

    const timeout = setTimeout(() => {
      reject(new Error("runtime response timeout"));
    }, 150);

    listeners.onRuntimeMessage(message, {}, (response) => {
      clearTimeout(timeout);
      resolve(response);
    });
  });
}

function waitForAsyncTasks() {
  return new Promise((resolve) => setTimeout(resolve, 20));
}

test("popup capture launches deep-link via tabs.update in active tab without opening extra tab", async () => {
  const { calls, listeners } = loadBackground();
  const response = await sendRuntimeMessage(listeners, { type: "capture_page" });

  assert.equal(response.ok, true);
  assert.equal(response.deepLink, "snorgnote://new?data=test");
  assert.equal(response.launchedInPage, true);
  assert.equal(calls.create.length, 0);
  assert.equal(calls.update.length, 1);
  assert.equal(calls.update[0].tabId, 7);
  assert.equal(calls.update[0].payload.url, "snorgnote://new?data=test");
  assert.equal(
    calls.sendMessage.some((call) => call.payload && call.payload.type === "open_deeplink"),
    false,
  );
  assert.equal(
    calls.sendMessage.some((call) => call.payload && call.payload.type === "extract_page"),
    true,
  );
});

test("popup capture returns error when tabs.update launch fails", async () => {
  const { listeners } = loadBackground({ updateError: new Error("update failed") });
  const response = await sendRuntimeMessage(listeners, { type: "capture_page" });

  assert.equal(response.ok, false);
  assert.equal(response.error, "Cannot launch Snorgnote from extension context.");
});

test("context menu capture opens extension launcher tab", async () => {
  const { calls, listeners } = loadBackground();
  assert.equal(typeof listeners.onContextMenuClicked, "function");

  listeners.onContextMenuClicked(
    {
      menuItemId: "snorgnote-send-selection",
      selectionText: "Selected from context menu",
    },
    {
      id: 77,
      url: "https://example.com/read",
      title: "Read page",
    },
  );

  await waitForAsyncTasks();

  assert.equal(calls.create.length, 1);
  assert.equal(calls.getURL.length >= 1, true);
  assert.equal(
    calls.create[0].url.startsWith(
      "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/launcher.html",
    ),
    true,
  );
  assert.equal(calls.create[0].url.includes("deeplink="), true);
  assert.equal(
    calls.sendMessage.some((call) => call.payload && call.payload.type === "open_deeplink"),
    false,
  );
});
