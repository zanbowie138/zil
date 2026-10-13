// The docs pages' zil examples: built to look like an editor (ec.config.mjs), swapped for CodeMirror when you point at or focus one,
// with Run (on the sandbox's wasm build) and Sandbox buttons.
import { call, encode, learn, warm } from "./client.js";
import "./cm.css";

// Once loaded, units and functions stand out in the editors.
let learned = null;
const vocab = () => (learned ??= warm().then(async () => learn(await call("vocab"))));
let cm = null;
const editor = () => (cm ??= import("./cm.js"));

/** A built block's code and results, as `[src, [[line, text], ...]]`. */
function read(code) {
  const src = [],
    rows = [];
  for (const line of code.querySelectorAll(".ec-line")) {
    const out = line.querySelector(".zil-out");
    if (out) {
      rows.push([src.length - 1, out.textContent]);
      continue;
    }
    const text = line.querySelector(".code");
    const inline = text.querySelector(".zil-inline");
    if (inline) rows.push([src.length, inline.title]);
    src.push([...text.childNodes].filter((n) => n !== inline).map((n) => n.textContent).join("").replace(/\n$/, ""));
  }
  return [src.join("\n"), rows];
}

for (const pre of document.querySelectorAll('.expressive-code pre[data-language="zil"]')) {
  const ex = pre.closest(".expressive-code");
  const status = ex.querySelector(".zil-status span");
  const hint = ex.querySelector(".zil-status span:last-child");
  const copy = ex.querySelector(".copy button");
  if (/Mac|iP/.test(navigator.platform) && hint) hint.textContent = "⌘+Enter to run";
  ex.classList.add("zil-run");

  const tools = document.createElement("div");
  tools.className = "zil-tools";
  tools.innerHTML = `<a class="btn" target="_blank" title="Open in the sandbox">Sandbox</a><button class="btn primary" title="Run this example (Ctrl+Enter while editing). Edit it first if you like.">Run</button>`;
  ex.append(tools);
  const [open, run] = tools.children;
  run.addEventListener("pointerenter", warm, { once: true });

  let view = null,
    mounting = null;
  const say = (s) => status && (status.textContent = s);
  const src = () => (view ? view.state.doc.toString() : read(pre.querySelector("code"))[0]);
  const link = () => (open.href = `/play/#code=${encode(src())}`);
  open.addEventListener("pointerdown", link);
  open.addEventListener("focus", link);

  /** Replaces the built code with an editor holding the same code and results. */
  const mount = () =>
    (mounting ??= editor().then(({ EditorState, EditorView, Prec, keymap, lineNumbers, relearn, setResults, zil }) => {
      const code = pre.querySelector("code");
      const [doc, rows] = read(code);
      view = new EditorView({
        state: EditorState.create({
          doc,
          extensions: [
            lineNumbers(),
            Prec.highest(keymap.of([{ key: "Mod-Enter", run: () => (go(), true) }])),
            zil({}),
            EditorView.contentAttributes.of({ "aria-label": "zil example" }),
            EditorView.updateListener.of((u) => {
              if (!u.docChanged) return;
              say("Edited");
              if (copy) copy.dataset.code = u.state.doc.toString();
            }),
          ],
        }),
      });
      view.dispatch({ effects: setResults.of(rows) });
      code.replaceWith(view.dom);
      pre.removeAttribute("tabindex");
      learned?.then(() => relearn(view));
      return { view, relearn, setResults };
    }));
  ex.addEventListener("pointerenter", mount, { once: true });
  // A click or Tab that lands before the editor is in still ends up in it, where it was aimed.
  pre.addEventListener("pointerdown", (e) => {
    if (view) return;
    mount().then(({ view }) => {
      view.focus();
      const at = view.posAtCoords({ x: e.clientX, y: e.clientY });
      if (at !== null) view.dispatch({ selection: { anchor: at } });
    });
  });
  pre.addEventListener("focus", () => view || mount().then(({ view }) => view.focus()));

  async function go() {
    if (ex.classList.contains("busy")) return;
    ex.classList.add("busy");
    run.textContent = "Running…";
    say("Running…");
    const { view, relearn, setResults } = await mount();
    const text = view.state.doc.toString();
    let rows;
    try {
      await vocab();
      relearn(view);
      const t0 = performance.now();
      rows = JSON.parse(await call("notebook", text));
      say(`Ran in ${Math.max(1, Math.round(performance.now() - t0))} ms`);
    } catch (e) {
      rows = [[view.state.doc.lines - 1, `error: ${e.message}`]];
      say("Failed");
    }
    view.dispatch({ effects: setResults.of(rows) });
    run.textContent = "Run";
    ex.classList.remove("busy");
  }
  run.addEventListener("click", go);
}
