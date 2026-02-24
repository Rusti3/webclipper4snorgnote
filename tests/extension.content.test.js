const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const CONTENT_PATH = path.join(__dirname, "..", "src", "extension", "content.js");

function loadContent() {
  let messageHandler = null;

  const document = {
    title: "Example",
    body: {
      innerText: "body text",
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

  return { handler: messageHandler };
}

test("content script returns selected text", () => {
  const { handler } = loadContent();
  assert.equal(typeof handler, "function");

  const responses = [];
  handler(
    { type: "extract_selection" },
    {},
    (response) => {
      responses.push(response);
    },
  );

  assert.equal(JSON.stringify(responses), JSON.stringify([{ selectionText: "selected" }]));
});

test("content script ignores deep-link launch messages", () => {
  const { handler } = loadContent();
  const responses = [];

  handler(
    { type: "open_deeplink", deepLink: "snorgnote://new?data=test" },
    {},
    (response) => {
      responses.push(response);
    },
  );

  assert.equal(responses.length, 0);
});
