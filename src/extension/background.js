importScripts("./payload.js");

const CONTEXT_MENU_ID = "snorgnote-send-selection";
const ERROR_STORAGE_KEY = "recentErrors";
const PENDING_POPUP_ACTION_KEY = "pendingPopupAction";
const MAX_ERROR_LOGS = 25;

function nowIso() {
  return new Date().toISOString();
}

function errorMessage(error) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error || "Unknown error");
}

async function saveErrorLog(context, error) {
  const message = errorMessage(error);
  const logEntry = {
    id: crypto.randomUUID(),
    context,
    message,
    at: nowIso(),
  };

  try {
    const current = await chrome.storage.local.get(ERROR_STORAGE_KEY);
    const prev = Array.isArray(current[ERROR_STORAGE_KEY]) ? current[ERROR_STORAGE_KEY] : [];
    const next = [logEntry, ...prev].slice(0, MAX_ERROR_LOGS);
    await chrome.storage.local.set({ [ERROR_STORAGE_KEY]: next });
  } catch {
    // Ignore storage failures to keep clipping flow alive.
  }
}

async function flashBadge(text, color) {
  try {
    await chrome.action.setBadgeBackgroundColor({ color });
    await chrome.action.setBadgeText({ text });
    setTimeout(() => {
      chrome.action.setBadgeText({ text: "" }).catch(() => {});
    }, 1800);
  } catch {
    // Ignore badge errors in background paths.
  }
}

async function ensureContextMenus() {
  await chrome.contextMenus.removeAll();
  chrome.contextMenus.create({
    id: CONTEXT_MENU_ID,
    title: "Send selected text to Snorgnote",
    contexts: ["selection"],
  });
}

function isHttpPage(url) {
  return typeof url === "string" && /^https?:\/\//i.test(url);
}

async function getActiveTab() {
  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  return tabs[0] || null;
}

function assertClippableTab(tab) {
  if (!tab || typeof tab.id !== "number") {
    throw new Error("No active tab found.");
  }
  if (!isHttpPage(tab.url)) {
    throw new Error("Open a regular http/https page to clip content.");
  }
}

async function requestContentExtraction(tabId, type) {
  try {
    const response = await chrome.tabs.sendMessage(tabId, { type });
    if (!response || typeof response !== "object") {
      throw new Error("Unexpected response from content script.");
    }
    return response;
  } catch {
    throw new Error("Cannot access page content. Refresh page and try again.");
  }
}

function markdownFromSelection(selectionText, pageUrl) {
  const body = selectionText.trim();
  if (!body) {
    return "";
  }
  const source = pageUrl ? `\n\n[Source](${pageUrl})` : "";
  return `${body}${source}`;
}

async function encodePayloadToDeepLink(payload) {
  if (!self.SnorgPayload || typeof self.SnorgPayload.clipPayloadToDeepLink !== "function") {
    throw new Error("Payload encoder is not available.");
  }
  return self.SnorgPayload.clipPayloadToDeepLink(payload);
}

function buildLauncherUrl(deepLink) {
  if (typeof deepLink !== "string" || !deepLink.trim()) {
    throw new Error("Deep-link payload is missing.");
  }
  const base = chrome.runtime.getURL("src/extension/launcher.html");
  const params = new URLSearchParams({
    deeplink: deepLink.trim(),
  });
  return `${base}?${params.toString()}`;
}

async function openLauncherTab(deepLink) {
  const launcherUrl = buildLauncherUrl(deepLink);
  await chrome.tabs.create({ url: launcherUrl });
}

async function dispatchClip(payload) {
  const encoded = await encodePayloadToDeepLink(payload);
  return {
    deepLink: encoded.deepLink,
    clipped: encoded.clipped,
    originalLength: encoded.originalLength,
    finalLength: encoded.finalLength,
  };
}

async function launchDeepLinkInTab(tabId, deepLink) {
  const normalized = typeof deepLink === "string" ? deepLink.trim() : "";
  if (!/^snorgnote:\/\//i.test(normalized)) {
    throw new Error("Generated deep-link is invalid.");
  }

  try {
    await chrome.tabs.update(tabId, { url: normalized });
  } catch {
    throw new Error("Cannot launch Snorgnote from extension context.");
  }
}

async function capturePage(tab) {
  assertClippableTab(tab);
  const extracted = await requestContentExtraction(tab.id, "extract_page");
  const contentMarkdown = typeof extracted.contentMarkdown === "string" ? extracted.contentMarkdown.trim() : "";

  if (!contentMarkdown) {
    throw new Error("Failed to extract readable content from this page.");
  }

  const payload = {
    type: "full_page",
    title: typeof extracted.title === "string" && extracted.title.trim() ? extracted.title.trim() : (tab.title || "Untitled"),
    url: typeof extracted.url === "string" && extracted.url.trim() ? extracted.url.trim() : (tab.url || ""),
    contentMarkdown,
    createdAt: nowIso(),
    source: "web-clipper",
  };

  return dispatchClip(payload);
}

async function captureSelection(tab, selectionFromMenu = "") {
  assertClippableTab(tab);
  let selectionText = selectionFromMenu.trim();

  if (!selectionText) {
    const extracted = await requestContentExtraction(tab.id, "extract_selection");
    selectionText = typeof extracted.selectionText === "string" ? extracted.selectionText.trim() : "";
  }

  if (!selectionText) {
    throw new Error("No selected text found.");
  }

  const payload = {
    type: "selection",
    title: tab.title || "Untitled",
    url: tab.url || "",
    contentMarkdown: markdownFromSelection(selectionText, tab.url || ""),
    createdAt: nowIso(),
    source: "web-clipper",
  };

  return dispatchClip(payload);
}

async function runClipAction(context, action) {
  try {
    const result = await action();
    return {
      ok: true,
      ...result,
    };
  } catch (error) {
    await saveErrorLog(context, error);
    return {
      ok: false,
      error: errorMessage(error),
    };
  }
}

async function handlePopupAction(actionType, options = {}) {
  const tab = await getActiveTab();
  if (actionType === "capture_page") {
    return runClipAction("capture_page", async () => {
      const result = await capturePage(tab);
      await launchDeepLinkInTab(tab.id, result.deepLink);
      return {
        ...result,
        launchedInPage: true,
      };
    });
  }

  if (actionType === "capture_selection") {
    return runClipAction("capture_selection", async () => {
      const selectionText = typeof options.selectionText === "string" ? options.selectionText : "";
      const result = await captureSelection(tab, selectionText);
      await launchDeepLinkInTab(tab.id, result.deepLink);
      return {
        ...result,
        launchedInPage: true,
      };
    });
  }

  return {
    ok: false,
    error: "Unknown popup action.",
  };
}

chrome.runtime.onInstalled.addListener(() => {
  ensureContextMenus().catch((error) => saveErrorLog("onInstalled", error));
});

chrome.runtime.onStartup.addListener(() => {
  ensureContextMenus().catch((error) => saveErrorLog("onStartup", error));
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId !== CONTEXT_MENU_ID) {
    return;
  }

  const pendingAction = {
    actionType: "capture_selection",
    selectionText: typeof info.selectionText === "string" ? info.selectionText : "",
    createdAt: nowIso(),
  };

  chrome.storage.local.set({ [PENDING_POPUP_ACTION_KEY]: pendingAction })
    .then(() => chrome.action.openPopup())
    .then(() => flashBadge("OK", "#16a34a"))
    .catch(async (error) => {
      await chrome.storage.local.remove(PENDING_POPUP_ACTION_KEY).catch(() => {});
      await saveErrorLog("context_menu_popup", error);
      flashBadge("ERR", "#dc2626");
    });
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (!message || typeof message.type !== "string") {
    return;
  }

  if (message.type === "capture_page" || message.type === "capture_selection") {
    handlePopupAction(message.type, message)
      .then(sendResponse)
      .catch(async (error) => {
        await saveErrorLog("popup_message", error);
        sendResponse({ ok: false, error: errorMessage(error) });
      });
    return true;
  }

  if (message.type === "get_recent_errors") {
    chrome.storage.local.get(ERROR_STORAGE_KEY)
      .then((store) => {
        const recentErrors = Array.isArray(store[ERROR_STORAGE_KEY]) ? store[ERROR_STORAGE_KEY] : [];
        sendResponse({ ok: true, recentErrors });
      })
      .catch((error) => {
        sendResponse({ ok: false, error: errorMessage(error), recentErrors: [] });
      });
    return true;
  }
});
