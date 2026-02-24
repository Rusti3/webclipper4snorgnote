const statusEl = document.getElementById("status");

function setStatus(message, kind = "") {
  if (!statusEl) {
    return;
  }
  statusEl.textContent = message;
  statusEl.className = kind;
}

function isValidDeepLink(value) {
  return typeof value === "string" && /^snorgnote:\/\//i.test(value.trim());
}

function readDeepLinkFromQuery() {
  const params = new URLSearchParams(window.location.search || "");
  const value = params.get("deeplink");
  return typeof value === "string" ? value.trim() : "";
}

function launch() {
  const deepLink = readDeepLinkFromQuery();
  if (!isValidDeepLink(deepLink)) {
    setStatus("Invalid launch data. Close this tab and try again.", "error");
    return;
  }

  setStatus("Launching Snorgnote...", "ok");
  window.location.href = deepLink;

  setTimeout(() => {
    try {
      window.close();
    } catch {
      setStatus("Snorgnote launch requested. You can close this tab.", "ok");
    }
  }, 1200);
}

launch();
