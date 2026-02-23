const readline = require("node:readline");

const rl = readline.createInterface({ input: process.stdin });

function send(payload) {
  process.stdout.write(`${JSON.stringify(payload)}\n`);
}

rl.on("line", (line) => {
  let msg;
  try {
    msg = JSON.parse(line);
  } catch {
    return;
  }

  const { id, cmd } = msg;
  send({ event: "progress", phase: cmd, message: "received" });

  if (cmd === "connect") {
    send({ id, ok: true, data: { status: "connected" } });
    return;
  }

  if (cmd === "import_urls") {
    const urls = Array.isArray(msg.urls) ? msg.urls : [];
    send({ id, ok: true, data: { imported: urls.length, failed: [] } });
    return;
  }

  if (cmd === "create_notebook") {
    const title = String(msg.title || "Auto Notebook");
    send({
      id,
      ok: true,
      data: {
        created: true,
        title,
        url: "https://notebooklm.google.com/notebook/mock-id",
      },
    });
    return;
  }

  if (cmd === "ask") {
    const prompt = String(msg.prompt || "");
    send({ id, ok: true, data: { answer: `mock answer for: ${prompt}` } });
    return;
  }

  if (cmd === "close") {
    send({ id, ok: true, data: { closed: true } });
    process.exit(0);
    return;
  }

  send({ id, ok: false, error: `unknown cmd: ${cmd}` });
});
