const statusEl = document.getElementById("status");
const clipPageButton = document.getElementById("clip-page");
const clipSelectionButton = document.getElementById("clip-selection");
const errorListEl = document.getElementById("error-list");

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

async function runCaptureAction(type) {
  setBusy(true);
  setStatus("", "Working...");

  try {
    const result = await chrome.runtime.sendMessage({ type });
    if (!result || !result.ok) {
      const message = result && typeof result.error === "string" ? result.error : "Clipping failed.";
      setStatus("error", message);
      await refreshRecentErrors();
      return;
    }

    setStatus("ok", `Sent successfully. clipId: ${result.clipId}`);
    await refreshRecentErrors();
  } catch (error) {
    setStatus("error", `Unexpected error: ${error.message || String(error)}`);
    await refreshRecentErrors();
  } finally {
    setBusy(false);
  }
}

clipPageButton.addEventListener("click", () => {
  runCaptureAction("capture_page");
});

clipSelectionButton.addEventListener("click", () => {
  runCaptureAction("capture_selection");
});

refreshRecentErrors();
