const statusEl = document.getElementById("status");
const clipPageButton = document.getElementById("clip-page");
const clipSelectionButton = document.getElementById("clip-selection");
const errorListEl = document.getElementById("error-list");
const PENDING_POPUP_ACTION_KEY = "pendingPopupAction";

function setBusy(isBusy) {
  clipPageButton.disabled = isBusy;
  clipSelectionButton.disabled = isBusy;
}

function setStatus(kind, text) {
  statusEl.textContent = text;
  statusEl.classList.remove("ok", "error");
  if (kind === "ok") {
    statusEl.classList.add("ok");
  }
  if (kind === "error") {
    statusEl.classList.add("error");
  }
}

function formatErrorEntry(entry) {
  if (!entry || typeof entry !== "object") {
    return "Unknown error";
  }

  const date = entry.at ? new Date(entry.at) : null;
  const dateText = date && !Number.isNaN(date.getTime()) ? date.toLocaleString() : "n/a";
  const context = typeof entry.context === "string" ? entry.context : "unknown";
  const message = typeof entry.message === "string" ? entry.message : "Unknown error";
  return `[${dateText}] (${context}) ${message}`;
}

async function refreshRecentErrors() {
  errorListEl.innerHTML = "";

  try {
    const response = await chrome.runtime.sendMessage({ type: "get_recent_errors" });
    const recentErrors = response && Array.isArray(response.recentErrors) ? response.recentErrors : [];

    if (!recentErrors.length) {
      const li = document.createElement("li");
      li.textContent = "No recent errors.";
      errorListEl.appendChild(li);
      return;
    }

    for (const entry of recentErrors) {
      const li = document.createElement("li");
      li.textContent = formatErrorEntry(entry);
      errorListEl.appendChild(li);
    }
  } catch (error) {
    const li = document.createElement("li");
    li.textContent = `Failed to load errors: ${error.message || String(error)}`;
    errorListEl.appendChild(li);
  }
}

async function runCaptureAction(type, options = {}) {
  setBusy(true);
  setStatus("", "Working...");

  try {
    const payload = { type };
    if (type === "capture_selection" && typeof options.selectionText === "string") {
      payload.selectionText = options.selectionText;
    }

    const result = await chrome.runtime.sendMessage(payload);
    if (!result || !result.ok) {
      const message = result && typeof result.error === "string" ? result.error : "Clipping failed.";
      setStatus("error", message);
      await refreshRecentErrors();
      return;
    }

    if (result.launchedInPage !== true) {
      setStatus("error", "Cannot confirm launch from extension context. Try again.");
      await refreshRecentErrors();
      return;
    }

    const clippedLabel = result.clipped ? " (clipped)" : "";
    setStatus("ok", `Launch requested from extension context${clippedLabel}.`);
    await refreshRecentErrors();
  } catch (error) {
    setStatus("error", `Unexpected error: ${error.message || String(error)}`);
    await refreshRecentErrors();
  } finally {
    setBusy(false);
  }
}

async function runPendingPopupAction() {
  if (!chrome.storage || !chrome.storage.local) {
    return;
  }

  let pending = null;
  try {
    const store = await chrome.storage.local.get(PENDING_POPUP_ACTION_KEY);
    pending = store ? store[PENDING_POPUP_ACTION_KEY] : null;
  } catch {
    return;
  }

  if (!pending || typeof pending !== "object") {
    return;
  }

  await chrome.storage.local.remove(PENDING_POPUP_ACTION_KEY).catch(() => {});
  if (pending.actionType !== "capture_selection") {
    return;
  }

  const selectionText = typeof pending.selectionText === "string" ? pending.selectionText : "";
  runCaptureAction("capture_selection", { selectionText });
}

clipPageButton.addEventListener("click", () => {
  runCaptureAction("capture_page");
});

clipSelectionButton.addEventListener("click", () => {
  runCaptureAction("capture_selection");
});

refreshRecentErrors();
runPendingPopupAction();
