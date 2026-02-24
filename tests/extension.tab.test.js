const test = require("node:test");
const assert = require("node:assert/strict");

const { isHttpPage, pickClippableTab } = require("../src/extension/tab");

test("isHttpPage returns true only for http/https urls", () => {
  assert.equal(isHttpPage("https://example.com"), true);
  assert.equal(isHttpPage("http://example.com"), true);
  assert.equal(isHttpPage("chrome://extensions"), false);
  assert.equal(isHttpPage("file:///C:/tmp/a.html"), false);
});

test("pickClippableTab prefers active tab when it is clippable", () => {
  const active = { id: 11, url: "https://active.test", lastAccessed: 1000 };
  const tabs = [
    active,
    { id: 12, url: "https://other.test", lastAccessed: 2000 },
  ];
  const picked = pickClippableTab(active, tabs);
  assert.equal(picked.id, 11);
});

test("pickClippableTab falls back to latest http tab when active tab is non-http", () => {
  const active = { id: 1, url: "chrome://extensions", lastAccessed: 5000 };
  const tabs = [
    active,
    { id: 2, url: "https://older.test", lastAccessed: 2000 },
    { id: 3, url: "https://latest.test", lastAccessed: 4000 },
  ];
  const picked = pickClippableTab(active, tabs);
  assert.equal(picked.id, 3);
});

test("pickClippableTab throws when there are no clippable tabs", () => {
  const active = { id: 1, url: "chrome://extensions" };
  assert.throws(() => pickClippableTab(active, [active]), /Open a regular http\/https page/);
});
