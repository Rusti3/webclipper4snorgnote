function isHttpPage(url) {
  return typeof url === "string" && /^https?:\/\//i.test(url);
}

function isClippableTab(tab) {
  return !!tab && typeof tab.id === "number" && isHttpPage(tab.url);
}

function pickClippableTab(activeTab, tabs) {
  if (isClippableTab(activeTab)) {
    return activeTab;
  }

  const list = Array.isArray(tabs) ? tabs : [];
  const candidates = list.filter(isClippableTab);
  if (!candidates.length) {
    const currentUrl = activeTab && typeof activeTab.url === "string" ? activeTab.url : "";
    const suffix = currentUrl ? ` Current tab: ${currentUrl}` : "";
    throw new Error(`Open a regular http/https page to clip content.${suffix}`);
  }

  candidates.sort((a, b) => {
    const aLast = typeof a.lastAccessed === "number" ? a.lastAccessed : 0;
    const bLast = typeof b.lastAccessed === "number" ? b.lastAccessed : 0;
    return bLast - aLast;
  });
  return candidates[0];
}

if (typeof self !== "undefined") {
  self.SnorgTab = {
    isHttpPage,
    pickClippableTab,
  };
}

if (typeof module !== "undefined") {
  module.exports = {
    isHttpPage,
    pickClippableTab,
  };
}
