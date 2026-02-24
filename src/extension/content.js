function normalizeWhitespace(text) {
  return text.replace(/\s+/g, " ").trim();
}

function escapedText(text) {
  return text
    .replace(/\\/g, "\\\\")
    .replace(/`/g, "\\`")
    .replace(/\*/g, "\\*")
    .replace(/_/g, "\\_")
    .replace(/\[/g, "\\[")
    .replace(/\]/g, "\\]");
}

function pickReadableRoot(doc) {
  const directCandidates = [doc.querySelector("article"), doc.querySelector("main")].filter(Boolean);
  for (const candidate of directCandidates) {
    if (normalizeWhitespace(candidate.innerText || "").length >= 200) {
      return candidate;
    }
  }

  let best = doc.body;
  let bestScore = 0;
  const candidates = doc.querySelectorAll("article,main,section,div");
  for (const candidate of candidates) {
    const text = normalizeWhitespace(candidate.innerText || "");
    const textLen = text.length;
    if (textLen < 200) {
      continue;
    }

    const pCount = candidate.querySelectorAll("p").length;
    const headingCount = candidate.querySelectorAll("h1,h2,h3").length;
    const linkTextLen = normalizeWhitespace(
      Array.from(candidate.querySelectorAll("a"), (node) => node.textContent || "").join(" ")
    ).length;
    const score = textLen + pCount * 140 + headingCount * 90 - linkTextLen * 0.4;

    if (score > bestScore) {
      bestScore = score;
      best = candidate;
    }
  }

  return best || doc.body;
}

function sanitizeRoot(node) {
  const clone = node.cloneNode(true);
  clone.querySelectorAll("script,style,noscript,svg,canvas,iframe,nav,header,footer,aside,form,button,input,textarea,select").forEach((el) => {
    el.remove();
  });
  clone.querySelectorAll("[hidden],[aria-hidden='true']").forEach((el) => {
    el.remove();
  });
  return clone;
}

function renderInline(node) {
  if (node.nodeType === Node.TEXT_NODE) {
    return escapedText(normalizeWhitespace(node.textContent || ""));
  }

  if (node.nodeType !== Node.ELEMENT_NODE) {
    return "";
  }

  const tag = node.tagName.toLowerCase();
  const content = renderInlineChildren(node);

  if (tag === "strong" || tag === "b") {
    return content ? `**${content}**` : "";
  }

  if (tag === "em" || tag === "i") {
    return content ? `*${content}*` : "";
  }

  if (tag === "code") {
    const value = normalizeWhitespace(node.textContent || "");
    return value ? `\`${escapedText(value)}\`` : "";
  }

  if (tag === "a") {
    const href = node.getAttribute("href") || "";
    const label = content || escapedText(normalizeWhitespace(node.textContent || ""));
    if (!label) {
      return "";
    }
    if (!href) {
      return label;
    }
    return `[${label}](${href})`;
  }

  if (tag === "img") {
    const src = node.getAttribute("src") || "";
    if (!src) {
      return "";
    }
    const alt = escapedText(node.getAttribute("alt") || "Image");
    return `![${alt}](${src})`;
  }

  if (tag === "br") {
    return "\n";
  }

  return content;
}

function renderInlineChildren(node) {
  const parts = [];
  for (const child of node.childNodes) {
    const next = renderInline(child);
    if (!next) {
      continue;
    }
    parts.push(next);
  }

  return parts
    .join(" ")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n[ \t]+/g, "\n")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
}

function indentLines(text, spaces) {
  const indent = " ".repeat(spaces);
  return text
    .split("\n")
    .map((line) => (line ? `${indent}${line}` : line))
    .join("\n");
}

function renderList(listNode, depth, ordered) {
  const items = Array.from(listNode.children).filter((el) => el.tagName && el.tagName.toLowerCase() === "li");
  const lines = [];

  items.forEach((li, index) => {
    const marker = ordered ? `${index + 1}.` : "-";
    const nested = [];
    const inlineParts = [];

    li.childNodes.forEach((child) => {
      if (child.nodeType === Node.ELEMENT_NODE) {
        const tag = child.tagName.toLowerCase();
        if (tag === "ul" || tag === "ol") {
          const nestedList = renderList(child, depth + 1, tag === "ol");
          if (nestedList) {
            nested.push(nestedList);
          }
          return;
        }
      }

      inlineParts.push(renderInline(child));
    });

    const text = inlineParts
      .join(" ")
      .replace(/[ \t]{2,}/g, " ")
      .trim();

    lines.push(`${"  ".repeat(depth)}${marker} ${text}`.trimEnd());

    nested.forEach((chunk) => {
      lines.push(indentLines(chunk, 2));
    });
  });

  return lines.join("\n");
}

function renderBlock(node, depth = 0) {
  if (node.nodeType === Node.TEXT_NODE) {
    return escapedText(normalizeWhitespace(node.textContent || ""));
  }

  if (node.nodeType !== Node.ELEMENT_NODE) {
    return "";
  }

  const tag = node.tagName.toLowerCase();

  if (tag === "h1" || tag === "h2" || tag === "h3" || tag === "h4" || tag === "h5" || tag === "h6") {
    const level = Number(tag.charAt(1));
    const content = renderInlineChildren(node);
    if (!content) {
      return "";
    }
    return `${"#".repeat(level)} ${content}`;
  }

  if (tag === "p") {
    return renderInlineChildren(node);
  }

  if (tag === "pre") {
    const code = (node.textContent || "").trimEnd();
    if (!code) {
      return "";
    }
    return `\`\`\`\n${code}\n\`\`\``;
  }

  if (tag === "blockquote") {
    const inner = Array.from(node.childNodes)
      .map((child) => renderBlock(child, depth + 1))
      .filter(Boolean)
      .join("\n")
      .trim();
    if (!inner) {
      return "";
    }

    return inner
      .split("\n")
      .map((line) => `> ${line}`)
      .join("\n");
  }

  if (tag === "ul" || tag === "ol") {
    return renderList(node, depth, tag === "ol");
  }

  if (tag === "table") {
    const rows = Array.from(node.querySelectorAll("tr"));
    if (!rows.length) {
      return "";
    }

    const markdownRows = rows.map((row) => {
      const cols = Array.from(row.children).map((cell) => renderInlineChildren(cell));
      return `| ${cols.join(" | ")} |`;
    });

    if (markdownRows.length > 1) {
      const colCount = Array.from(rows[0].children).length || 1;
      const separator = `| ${Array.from({ length: colCount }, () => "---").join(" | ")} |`;
      markdownRows.splice(1, 0, separator);
    }

    return markdownRows.join("\n");
  }

  if (tag === "hr") {
    return "---";
  }

  if (tag === "br") {
    return "";
  }

  const parts = [];
  for (const child of node.childNodes) {
    const block = renderBlock(child, depth);
    if (!block) {
      continue;
    }
    parts.push(block);
  }

  return parts.join("\n");
}

function htmlToMarkdown(root) {
  const sections = [];
  for (const child of root.childNodes) {
    const block = renderBlock(child, 0);
    if (!block) {
      continue;
    }
    sections.push(block.trim());
  }

  return sections
    .join("\n\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function extractPage() {
  const title = document.title || "Untitled";
  const url = location.href;
  const root = pickReadableRoot(document);
  const sanitizedRoot = sanitizeRoot(root);
  const markdown = htmlToMarkdown(sanitizedRoot);

  if (markdown) {
    return {
      title,
      url,
      contentMarkdown: markdown
    };
  }

  const fallback = normalizeWhitespace(document.body ? document.body.innerText || "" : "");
  return {
    title,
    url,
    contentMarkdown: fallback
  };
}

function extractSelection() {
  const selection = window.getSelection();
  const text = selection ? selection.toString() : "";
  return text.trim();
}

function isSnorgnoteDeepLink(deepLink) {
  return typeof deepLink === "string" && /^snorgnote:\/\//i.test(deepLink.trim());
}

function launchDeepLinkOnPage(deepLink) {
  if (!isSnorgnoteDeepLink(deepLink)) {
    throw new Error("Invalid deep-link URL.");
  }

  const host = document.body || document.documentElement;
  if (!host || typeof host.appendChild !== "function") {
    throw new Error("Cannot launch Snorgnote from this page.");
  }

  const anchor = document.createElement("a");
  anchor.href = deepLink.trim();
  anchor.rel = "noopener noreferrer";
  anchor.target = "_self";

  host.appendChild(anchor);
  try {
    anchor.click();
  } finally {
    if (typeof host.removeChild === "function") {
      host.removeChild(anchor);
    }
  }
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (!message || typeof message.type !== "string") {
    return;
  }

  if (message.type === "extract_page") {
    sendResponse(extractPage());
    return;
  }

  if (message.type === "extract_selection") {
    sendResponse({ selectionText: extractSelection() });
    return;
  }

  if (message.type === "open_deeplink") {
    try {
      launchDeepLinkOnPage(message.deepLink);
      sendResponse({ ok: true });
    } catch (error) {
      const details = error && typeof error.message === "string"
        ? error.message
        : "Failed to launch application.";
      sendResponse({ ok: false, error: details });
    }
  }
});
