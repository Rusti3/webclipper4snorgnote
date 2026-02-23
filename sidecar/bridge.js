const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const readline = require("node:readline");
const { chromium } = require("playwright");

const rl = readline.createInterface({ input: process.stdin });

let ctx = null;
let page = null;
let options = parseArgs(process.argv.slice(2));

function parseArgs(argv) {
  const parsed = {
    profileDir: path.join(os.homedir(), ".myapp", "notebooklm-profile"),
    browserPath: "",
  };
  for (let i = 0; i < argv.length; i++) {
    const token = argv[i];
    if (token === "--profile-dir" && argv[i + 1]) {
      parsed.profileDir = argv[++i];
      continue;
    }
    if (token === "--browser-path" && argv[i + 1]) {
      parsed.browserPath = argv[++i];
      continue;
    }
  }
  return parsed;
}

function send(payload) {
  process.stdout.write(`${JSON.stringify(payload)}\n`);
}

function emitProgress(phase, message, extra = {}) {
  send({ event: "progress", phase, message, ...extra });
}

function ok(id, data) {
  send({ id, ok: true, data });
}

function fail(id, error) {
  send({ id, ok: false, error: String(error && error.message ? error.message : error) });
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function guessBrowserPath() {
  const candidates = [
    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
    "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return "";
}

async function ensureContext() {
  if (ctx && page) {
    return;
  }
  fs.mkdirSync(options.profileDir, { recursive: true });
  const executablePath = options.browserPath || guessBrowserPath() || undefined;

  ctx = await chromium.launchPersistentContext(options.profileDir, {
    headless: false,
    executablePath,
    args: ["--disable-blink-features=AutomationControlled"],
    viewport: { width: 1400, height: 960 },
  });
  page = ctx.pages()[0] || (await ctx.newPage());
}

async function openNotebookLM() {
  await ensureContext();
  await page.goto("https://notebooklm.google.com/", { waitUntil: "domcontentloaded" });
}

async function locatorVisible(locator, timeoutMs = 1200) {
  try {
    await locator.first().waitFor({ state: "visible", timeout: timeoutMs });
    return true;
  } catch {
    return false;
  }
}

async function clickCreateNotebookButton() {
  const candidates = [
    () => page.getByRole("button", { name: /new notebook/i }),
    () => page.getByText(/new notebook/i),
    () => page.locator('[aria-label*="new notebook" i]'),
    () => page.locator("button:has-text('New notebook')"),
  ];

  for (const candidate of candidates) {
    try {
      const locator = candidate().first();
      if (await locatorVisible(locator, 1500)) {
        await locator.click({ timeout: 5000 });
        return true;
      }
    } catch {
      // no-op
    }
  }
  return false;
}

async function clickTabByName(pattern) {
  const candidates = [
    () => page.getByRole("tab", { name: pattern }),
    () => page.getByRole("button", { name: pattern }),
    () => page.getByText(pattern),
  ];

  for (const candidate of candidates) {
    try {
      const locator = candidate().first();
      if (await locatorVisible(locator, 1000)) {
        await locator.click({ timeout: 4000 });
        return true;
      }
    } catch {
      // no-op
    }
  }
  return false;
}

async function ensureAddSourceInputOpen() {
  const inputCandidates = [
    () => page.locator('input[type="url"]'),
    () => page.locator('input[placeholder*="paste" i]'),
    () => page.locator('input[placeholder*="link" i]'),
    () => page.locator('input[type="text"]'),
  ];

  for (const inputCandidate of inputCandidates) {
    if (await locatorVisible(inputCandidate(), 800)) {
      return inputCandidate().first();
    }
  }

  const openCandidates = [
    () => page.getByRole("button", { name: /add source/i }),
    () => page.getByText(/add source/i),
    () => page.getByRole("button", { name: /website/i }),
    () => page.getByText(/website/i),
  ];

  for (const openCandidate of openCandidates) {
    try {
      const btn = openCandidate().first();
      if (await locatorVisible(btn, 1000)) {
        await btn.click({ timeout: 4000 });
        await sleep(400);
        break;
      }
    } catch {
      // no-op
    }
  }

  for (const inputCandidate of inputCandidates) {
    const input = inputCandidate().first();
    if (await locatorVisible(input, 1500)) {
      return input;
    }
  }
  throw new Error("cannot find source input field");
}

async function setNotebookTitle(title) {
  const titleCandidates = [
    () => page.locator('input[aria-label*="title" i]'),
    () => page.locator('textarea[aria-label*="title" i]'),
    () => page.locator('input[type="text"]'),
  ];

  for (const candidate of titleCandidates) {
    try {
      const box = candidate().first();
      if (await locatorVisible(box, 1500)) {
        await box.fill(title);
        await page.keyboard.press("Enter");
        return true;
      }
    } catch {
      // no-op
    }
  }
  return false;
}

async function waitForLoginReady(timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await clickCreateNotebookButton()) {
      await page.goBack().catch(() => {});
      return true;
    }
    const hasCreateButton = await locatorVisible(page.getByText(/new notebook/i).first(), 500);
    if (hasCreateButton) {
      return true;
    }
    await sleep(1000);
  }
  return false;
}

async function findChatInput() {
  const candidates = [
    () => page.locator("textarea"),
    () => page.locator('[contenteditable="true"]'),
    () => page.locator('div[role="textbox"]'),
  ];
  for (const candidate of candidates) {
    const locator = candidate().first();
    if (await locatorVisible(locator, 1500)) {
      return locator;
    }
  }
  return null;
}

function trimLongText(text, limit) {
  if (!text || text.length <= limit) {
    return text || "";
  }
  return text.slice(0, limit);
}

async function extractChatAnswer(beforeText, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let latest = beforeText;
  while (Date.now() < deadline) {
    const bodyText = await page.locator("body").innerText();
    if (bodyText.length > latest.length) {
      latest = bodyText;
    }
    if (latest.length > beforeText.length + 20) {
      await sleep(1500);
      break;
    }
    await sleep(1000);
  }

  let answer = latest;
  if (latest.startsWith(beforeText)) {
    answer = latest.slice(beforeText.length);
  }
  answer = answer.trim();
  if (!answer) {
    return "";
  }
  return trimLongText(answer, 16000);
}

async function handleConnect(id) {
  emitProgress("connect", "opening notebooklm");
  await openNotebookLM();
  emitProgress("connect", "waiting for login or ready state");
  const ready = await waitForLoginReady(3 * 60 * 1000);
  ok(id, {
    status: ready ? "connected" : "connected_or_timeout",
    url: page.url(),
  });
}

async function handleCreateNotebook(id, msg) {
  const title = String(msg.title || "Auto Notebook");
  emitProgress("create", "opening notebook home");
  await openNotebookLM();

  const clicked = await clickCreateNotebookButton();
  if (!clicked) {
    throw new Error("cannot find New notebook button");
  }
  await sleep(1200);

  await setNotebookTitle(title).catch(() => {});
  const url = page.url();

  ok(id, {
    created: true,
    title,
    url,
  });
}

async function handleImportUrls(id, msg) {
  const urls = Array.isArray(msg.urls) ? msg.urls : [];
  emitProgress("import", "starting import", { current: 0, total: urls.length });

  await clickTabByName(/sources/i).catch(() => {});
  const failed = [];
  let imported = 0;

  for (let i = 0; i < urls.length; i++) {
    const url = String(urls[i] || "").trim();
    emitProgress("import", "importing url", {
      current: i + 1,
      total: urls.length,
      url,
    });

    if (!url) {
      failed.push({ url, reason: "empty url" });
      continue;
    }

    try {
      const input = await ensureAddSourceInputOpen();
      await input.fill(url);
      await page.keyboard.press("Enter");
      await sleep(900);
      imported += 1;
    } catch (error) {
      failed.push({ url, reason: String(error && error.message ? error.message : error) });
    }
  }

  ok(id, { imported, failed });
}

async function handleAsk(id, msg) {
  const prompt = String(msg.prompt || "").trim();
  if (!prompt) {
    throw new Error("empty prompt");
  }

  await clickTabByName(/chat/i).catch(() => {});
  const input = await findChatInput();
  if (!input) {
    throw new Error("cannot find chat input");
  }

  const beforeText = await page.locator("body").innerText();
  await input.click({ timeout: 4000 });

  const editable = await input.evaluate((node) => node.getAttribute("contenteditable"));
  if (editable && editable.toLowerCase() === "true") {
    await page.keyboard.type(prompt, { delay: 10 });
  } else {
    await input.fill(prompt);
  }

  await page.keyboard.press("Enter");
  emitProgress("ask", "prompt submitted");
  const answer = await extractChatAnswer(beforeText, 90 * 1000);

  ok(id, { answer });
}

async function handleClose(id) {
  if (ctx) {
    await ctx.close().catch(() => {});
  }
  ctx = null;
  page = null;
  ok(id, { closed: true });
}

rl.on("line", async (line) => {
  let msg;
  try {
    msg = JSON.parse(line);
  } catch {
    return;
  }

  const { id, cmd } = msg;
  if (!id || !cmd) {
    return;
  }

  try {
    if (cmd === "connect") {
      await handleConnect(id);
      return;
    }
    if (cmd === "create_notebook") {
      await handleCreateNotebook(id, msg);
      return;
    }
    if (cmd === "import_urls") {
      await handleImportUrls(id, msg);
      return;
    }
    if (cmd === "ask") {
      await handleAsk(id, msg);
      return;
    }
    if (cmd === "close") {
      await handleClose(id);
      return;
    }
    fail(id, `unknown cmd: ${cmd}`);
  } catch (error) {
    fail(id, error);
  }
});
