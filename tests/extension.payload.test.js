const test = require("node:test");
const assert = require("node:assert/strict");

const {
  DEFAULT_MAX_MARKDOWN_CHARS,
  clipPayloadToDeepLink,
  truncateMarkdown,
} = require("../src/extension/payload");

test("truncateMarkdown keeps short content intact", () => {
  const input = "short";
  const result = truncateMarkdown(input, 10);
  assert.equal(result.content, "short");
  assert.equal(result.clipped, false);
});

test("truncateMarkdown clips long content and appends marker", () => {
  const input = "A".repeat(50);
  const result = truncateMarkdown(input, 10);
  assert.equal(result.clipped, true);
  assert.match(result.content, /\[CLIPPED:/);
});

test("clipPayloadToDeepLink builds snorgnote data URI", () => {
  const payload = {
    type: "selection",
    title: "Hello",
    url: "https://example.com",
    contentMarkdown: "x",
    createdAt: "2026-02-24T00:00:00.000Z",
    source: "web-clipper",
  };

  const { deepLink } = clipPayloadToDeepLink(payload, DEFAULT_MAX_MARKDOWN_CHARS);
  assert.match(deepLink, /^snorgnote:\/\/new\?data=/);
});
