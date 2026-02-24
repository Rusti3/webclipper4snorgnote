const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const LAUNCHER_PATH = path.join(__dirname, "..", "src", "extension", "launcher.js");

function loadLauncher(search) {
  const status = {
    textContent: "",
    className: "",
  };

  const document = {
    getElementById(id) {
      if (id === "status") {
        return status;
      }
      return null;
    },
  };

  let closeCalls = 0;
  const window = {
    location: {
      href: "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/launcher.html",
      search,
    },
    close() {
      closeCalls += 1;
    },
  };

  const context = {
    console,
    document,
    window,
    URLSearchParams,
    setTimeout(handler) {
      handler();
      return 1;
    },
    clearTimeout() {},
  };

  vm.createContext(context);
  vm.runInContext(fs.readFileSync(LAUNCHER_PATH, "utf8"), context, { filename: "launcher.js" });

  return { status, window, closeCalls: () => closeCalls };
}

test("launcher redirects to snorgnote deep-link and closes itself", () => {
  const { window, closeCalls } = loadLauncher("?deeplink=snorgnote%3A%2F%2Fnew%3Fdata%3Dabc");
  assert.equal(window.location.href, "snorgnote://new?data=abc");
  assert.equal(closeCalls(), 1);
});

test("launcher shows error for invalid deep-link", () => {
  const { status, window, closeCalls } = loadLauncher("?deeplink=https%3A%2F%2Fexample.com");
  assert.equal(
    window.location.href,
    "chrome-extension://cnjifjpddelmedmihgijeibhnjfabmlf/src/extension/launcher.html",
  );
  assert.equal(status.className.includes("error"), true);
  assert.equal(closeCalls(), 0);
});
