const DEEP_LINK_BASE = "snorgnote://new";
const DEFAULT_MAX_MARKDOWN_CHARS = 12000;

function toBase64UrlUtf8(value) {
  const json = JSON.stringify(value);
  const bytes = new TextEncoder().encode(json);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function truncateMarkdown(markdown, maxChars = DEFAULT_MAX_MARKDOWN_CHARS) {
  const value = String(markdown || "");
  if (value.length <= maxChars) {
    return {
      content: value,
      clipped: false,
      originalLength: value.length,
    };
  }

  const clippedContent = value.slice(0, maxChars);
  const marker = `\n\n[CLIPPED: original_length=${value.length} chars, limit=${maxChars} chars]`;
  return {
    content: `${clippedContent}${marker}`,
    clipped: true,
    originalLength: value.length,
  };
}

function clipPayloadToDeepLink(payload, maxChars = DEFAULT_MAX_MARKDOWN_CHARS) {
  if (!payload || typeof payload !== "object") {
    throw new Error("Invalid clip payload.");
  }
  const required = ["type", "title", "url", "contentMarkdown", "createdAt"];
  for (const key of required) {
    if (typeof payload[key] !== "string" || payload[key].trim() === "") {
      throw new Error(`Missing or invalid field: ${key}.`);
    }
  }

  const clipped = truncateMarkdown(payload.contentMarkdown, maxChars);
  const normalized = {
    type: payload.type.trim(),
    title: payload.title.trim(),
    url: payload.url.trim(),
    contentMarkdown: clipped.content.trim(),
    createdAt: payload.createdAt.trim(),
    source: typeof payload.source === "string" && payload.source.trim()
      ? payload.source.trim()
      : "web-clipper",
  };

  const data = toBase64UrlUtf8(normalized);
  return {
    deepLink: `${DEEP_LINK_BASE}?data=${data}`,
    clipped: clipped.clipped,
    originalLength: clipped.originalLength,
    finalLength: normalized.contentMarkdown.length,
  };
}

if (typeof self !== "undefined") {
  self.SnorgPayload = {
    DEFAULT_MAX_MARKDOWN_CHARS,
    clipPayloadToDeepLink,
    truncateMarkdown,
  };
}

if (typeof module !== "undefined") {
  module.exports = {
    DEFAULT_MAX_MARKDOWN_CHARS,
    clipPayloadToDeepLink,
    truncateMarkdown,
  };
}
