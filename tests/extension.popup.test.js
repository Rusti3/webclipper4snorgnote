const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const POPUP_PATH = path.join(__dirname, "..", "src", "extension", "popup.js");

function createClassList() {
  const classes = new Set();
  return {
    add(name) {
      classes.add(name);
    },
    remove(...names) {
      for (const name of names) {
        classes.delete(name);
      }
    },
    has(name) {
      return classes.has(name);
    },
  };
}

function createElement() {
  const handlers = {};
  return {
    disabled: false,
    textContent: "",
    classList: createClassList(),
    innerHTML: "",
    appendChild() {},
    addEventListener(type, handler) {
      handlers[type] = handler;
    },
    fire(type) {
      if (typeof handlers[type] === "function") {
        handlers[type]();
      }
    },
  };
}

function loadPopup(sendMessageImpl) {
  const elements = {
    status: createElement(),
    "clip-page": createElement(),
    "clip-selection": createElement(),
    "error-list": createElement(),
  };

  const document = {
    createElement() {
      return createElement();
    },
    getElementById(id) {
      return elements[id];
    },
  };

  const window = {
    location: {
      href: "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/popup.html",
    },
  };

  const chrome = {
    runtime: {
      async sendMessage(payload) {
        return sendMessageImpl(payload);
      },
    },
  };

  const context = {
    console,
    chrome,
    document,
    window,
    setTimeout,
    clearTimeout,
  };

  vm.createContext(context);
  vm.runInContext(fs.readFileSync(POPUP_PATH, "utf8"), context, { filename: "popup.js" });

  return { elements, window };
}

async function flush() {
  await new Promise((resolve) => setTimeout(resolve, 10));
}

test("popup capture stays on popup and reports launch in current page", async () => {
  const popupUrl = "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/popup.html";
  const { elements, window } = loadPopup(async (payload) => {
    if (payload.type === "get_recent_errors") {
      return { ok: true, recentErrors: [] };
    }
    if (payload.type === "capture_page") {
      return {
        ok: true,
        clipped: false,
        launchedInPage: true,
      };
    }
    return { ok: false, error: "unexpected" };
  });

  elements["clip-page"].fire("click");
  await flush();

  assert.equal(window.location.href, popupUrl);
  assert.equal(elements.status.classList.has("ok"), true);
});

test("popup capture shows error when background returns launch failure", async () => {
  const { elements, window } = loadPopup(async (payload) => {
    if (payload.type === "get_recent_errors") {
      return { ok: true, recentErrors: [] };
    }
    if (payload.type === "capture_selection") {
      return {
        ok: false,
        error: "Cannot launch Snorgnote from this page.",
      };
    }
    return { ok: false, error: "unexpected" };
  });

  elements["clip-selection"].fire("click");
  await flush();

  assert.equal(
    window.location.href,
    "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/popup.html",
  );
  assert.equal(elements.status.classList.has("error"), true);
});
