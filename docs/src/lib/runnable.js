// The docs pages' zil examples: editable in place, with Run (on the sandbox's wasm build) and Sandbox buttons.
import { call, encode, highlight, learn, warm } from "./client.js";

const RESULT = /^\s*# → /;
/** The block without its result lines: what the reader wrote. */
const source = (text) =>
  text
    .replace(/\n$/, "")
    .split("\n")
    .filter((l) => !RESULT.test(l))
    .join("\n");
/** `src` with each `[line, text]` from `notebook` written as `# → ` lines after its line. */
function merge(src, rows) {
  const after = new Map(rows);
  return src
    .split("\n")
    .flatMap((line, i) => (after.has(i) ? [line, ...after.get(i).split("\n").map((r) => `# → ${r}`)] : [line]))
    .join("\n");
}
/** Code as Expressive Code's line structure, so its padding and line styles still apply. */
const lines = (text) => text.split("\n").map((l) => `<div class="ec-line"><div class="code">${highlight(l) || "<br>"}</div></div>`).join("");

// Once loaded, units and functions stand out in re-highlighted blocks.
let learned = null;
const vocab = () => (learned ??= warm().then(async () => learn(await call("vocab"))));

for (const pre of document.querySelectorAll('.expressive-code pre[data-language="zil"]')) {
  const ex = pre.closest(".expressive-code");
  const code = pre.querySelector("code");
  const copy = ex.querySelector(".copy button");
  ex.classList.add("zil-run");
  code.contentEditable = "plaintext-only";
  code.spellcheck = false;
  const text = () => code.innerText;

  const tools = document.createElement("div");
  tools.className = "zil-tools";
  tools.innerHTML = `<a class="btn" target="_blank" title="Open in the sandbox">Sandbox</a><button class="btn primary" title="Run this example (Ctrl+Enter while editing). Edit it first if you like.">Run</button>`;
  ex.append(tools);
  const [open, run] = tools.children;
  run.addEventListener("pointerenter", warm, { once: true });
  const link = () => (open.href = `/play/#code=${encode(source(text()))}`);
  open.addEventListener("pointerdown", link);
  open.addEventListener("focus", link);

  async function go() {
    if (ex.classList.contains("busy")) return;
    ex.classList.add("busy");
    run.textContent = "Running…";
    const src = source(text());
    let rows;
    try {
      await vocab();
      rows = JSON.parse(await call("notebook", src));
    } catch (e) {
      rows = [[src.split("\n").length - 1, e.message]];
    }
    const out = merge(src, rows);
    code.innerHTML = lines(out);
    if (copy) copy.dataset.code = out;
    run.textContent = "Run";
    ex.classList.remove("busy");
  }
  run.addEventListener("click", go);
  code.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      go();
    }
  });
}
