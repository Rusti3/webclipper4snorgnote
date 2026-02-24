const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const CONTENT_PATH = path.join(__dirname, "..", "src", "extension", "content.js");

function loadContent() {
  const calls = {
    appendChild: [],
    removeChild: [],
    click: 0,
  };

  let messageHandler = null;
  const anchor = {
    href: "",
    rel: "",
    target: "",
    click() {
      calls.click += 1;
    },
  };

  const document = {
    title: "Example",
    body: {
      innerText: "body text",
      appendChild(node) {
        calls.appendChild.push(node);
      },
      removeChild(node) {
        calls.removeChild.push(node);
      },
    },
    createElement(tagName) {
      if (tagName !== "a") {
        throw new Error(`unexpected tag: ${tagName}`);
      }
      return anchor;
    },
  };

  const chrome = {
    runtime: {
      onMessage: {
        addListener(handler) {
          messageHandler = handler;
        },
      },
    },
  };

  const context = {
    console,
    chrome,
    document,
    location: { href: "https://example.com/articles/1" },
    window: {
      getSelection() {
        return {
          toString() {
            return "selected";
          },
        };
      },
    },
    Node: {
      TEXT_NODE: 3,
      ELEMENT_NODE: 1,
    },
  };

  vm.createContext(context);
  vm.runInContext(fs.readFileSync(CONTENT_PATH, "utf8"), context, { filename: "content.js" });

  return { handler: messageHandler, calls, anchor };
}

test("content script handles open_deeplink without page navigation", () => {
  const { handler, calls, anchor } = loadContent();
  assert.equal(typeof handler, "function");

  const responses = [];
  handler(
    { type: "open_deeplink", deepLink: "snorgnote://new?data=test" },
    {},
    (response) => {
      responses.push(response);
    },
  );

  assert.equal(calls.click, 1);
  assert.equal(calls.appendChild.length, 1);
  assert.equal(calls.removeChild.length, 1);
  assert.equal(anchor.href, "snorgnote://new?data=test");
  assert.equal(JSON.stringify(responses), JSON.stringify([{ ok: true }]));
});

test("content script rejects non-snorgnote deep links", () => {
  const { handler, calls } = loadContent();
  const responses = [];

  handler(
    { type: "open_deeplink", deepLink: "https://example.com" },
    {},
    (response) => {
      responses.push(response);
    },
  );

  assert.equal(calls.click, 0);
  assert.equal(
    JSON.stringify(responses),
    JSON.stringify([{ ok: false, error: "Invalid deep-link URL." }]),
  );
});
