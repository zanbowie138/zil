// The sandbox page: a REPL and a script editor on the wasm build, a searchable example catalog, and a first-visit tour.
// Bundled into docs/sandbox/app.js by `pnpm js`.
import { completionStatus } from "@codemirror/autocomplete";
import { insertNewlineAndIndent } from "@codemirror/commands";
import { highlightActiveLine, highlightActiveLineGutter, lineNumbers, placeholder } from "@codemirror/view";
import { call, decode, encode, escape, generation, highlight, learn, stop, vocab, warm } from "../sandbox/client.js";
import { EditorState, EditorView, Prec, keymap, relearn, setResults, zil } from "./cm.js";

const START = `# zil: a calculator language with units, dates and exact math.
# Each line shows its value as you type. Change a number and watch.

trip = 42 km
trip to mi
trip / 65 mph to min
date("2026-12-25 18:30") to "Tokyo"
0.1 + 0.2 == 0.3

[3, 1, 2] |> sort |> map(|x| x ** 2)
`;
const $ = (id) => document.getElementById(id);
/** localStorage, which can throw in private windows and previews. */
const store = {
  get: (k) => {
    try {
      return localStorage.getItem(k);
    } catch {
      return null;
    }
  },
  set: (k, v) => {
    try {
      localStorage.setItem(k, v);
    } catch {}
  },
};
const mac = /Mac|iPhone|iPad/.test(navigator.platform);
if (mac) $("key").textContent = "⌘ ↵";

// ── Script mode ──────────────────────────────────────────────────────────────

const out = $("out"),
  status = $("status"),
  runBtn = $("run"),
  live = $("live");
const SAVED = "zil.sandbox";

const editor = new EditorView({
  parent: $("editor"),
  state: EditorState.create({
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      Prec.highest(keymap.of([{ key: "Mod-Enter", run: () => (run(), true) }])),
      zil({ onHelp: (name) => helpFor(name) }),
      EditorView.contentAttributes.of({ "aria-label": "zil code" }),
      EditorView.updateListener.of((u) => u.docChanged && edited()),
    ],
  }),
});
const code = () => editor.state.doc.toString();

let timer;
function edited() {
  for (const b of document.querySelectorAll(".try[aria-current]")) b.removeAttribute("aria-current");
  store.set(SAVED, code());
  clearTimeout(timer);
  if (live.checked) timer = setTimeout(run, 350);
}

/** Replaces the whole script, undoably, and runs it. */
function load(src) {
  editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: src }, selection: { anchor: 0 }, scrollIntoView: true });
  editor.focus();
  run();
}

/** Adds `src` on a new line after the cursor's line, so a snippet never wipes out what you wrote. */
function insert(src) {
  const line = editor.state.doc.lineAt(editor.state.selection.main.head);
  const text = (line.text.trim() ? "\n" : "") + src;
  editor.dispatch({ changes: { from: line.to, insert: text }, selection: { anchor: line.to + text.length }, scrollIntoView: true });
  editor.focus();
  run();
}

let running = false,
  again = false;
async function run() {
  if (running) return void (again = true);
  running = true;
  runBtn.firstChild.textContent = "Stop ";
  status.classList.add("busy");
  status.textContent = "Running";
  const t0 = performance.now();
  try {
    const [printed, result, error, lines] = await call("run", code());
    editor.dispatch({ effects: setResults.of(JSON.parse(lines)) });
    out.innerHTML =
      (printed ? `<pre class="printed">${printed}</pre>` : "") +
        (result ? `<div class="result"><pre>${result}</pre></div>` : "") +
        (error ? `<div class="error"><pre>${error}</pre></div>` : "") ||
      `<p class="note">Nothing printed. Each line's value shows beside it in the editor; anything you <code>print</code>, and the last line's value, shows here.</p>`;
    status.textContent = `${Math.max(1, Math.round(performance.now() - t0))} ms`;
  } catch (e) {
    out.innerHTML = `<div class="error"><pre>${escape(e.message)}</pre></div>`;
    status.textContent = "";
  } finally {
    running = false;
    status.classList.remove("busy");
    runBtn.firstChild.textContent = "Run ";
    if (again) {
      again = false;
      run();
    }
  }
}

runBtn.addEventListener("click", () => (running ? stop() : run()));
live.checked = store.get("zil.live") !== "false";
live.addEventListener("change", () => {
  store.set("zil.live", live.checked);
  if (live.checked) run();
});

$("share").addEventListener("click", async (e) => {
  const url = `${location.origin}${location.pathname}#code=${encode(code())}`;
  history.replaceState(null, "", url);
  try {
    await navigator.clipboard.writeText(url);
    e.target.textContent = "Link copied";
  } catch {
    e.target.textContent = "Link in address bar";
  }
  setTimeout(() => (e.target.textContent = "Share"), 1600);
});

// ── REPL mode: one session in the worker, like the terminal REPL, with the prompt as the transcript's last line ──

const log = $("log"),
  prompt = $("prompt"),
  rstatus = $("rstatus");
const PAST = "zil.history";
let past = [];
try {
  past = JSON.parse(store.get(PAST)) ?? [];
} catch {}
let pos = past.length,
  draft = "",
  busy = false;
/** The worker generation whose session we greeted; another one means the session was lost. */
let session = 0;

const popup = (view) => completionStatus(view.state) === "active";
const input = new EditorView({
  parent: $("input"),
  state: EditorState.create({
    extensions: [
      Prec.highest(
        keymap.of([
          { key: "Enter", run: enter },
          { key: "Mod-Enter", run: () => (busy || submit(), true) },
          { key: "Shift-Enter", run: insertNewlineAndIndent },
          { key: "ArrowUp", run: (v) => !popup(v) && v.state.doc.lineAt(v.state.selection.main.head).number === 1 && older() },
          { key: "ArrowDown", run: (v) => !popup(v) && v.state.doc.lineAt(v.state.selection.main.head).number === v.state.doc.lines && newer() },
          { key: "Ctrl-l", run: () => (clearLog(), true) },
          { key: "Ctrl-c", run: (v) => busy && v.state.selection.main.empty && (stop(), true) },
        ]),
      ),
      zil({ onHelp: (name) => helpFor(name) }),
      placeholder("try help(), or type a few letters and press Tab"),
      EditorView.lineWrapping,
      EditorView.contentAttributes.of({ "aria-label": "REPL input" }),
    ],
  }),
});
const typed = () => input.state.doc.toString();

function setInput(src) {
  input.dispatch({ changes: { from: 0, to: input.state.doc.length, insert: src }, selection: { anchor: src.length } });
}
function older() {
  if (pos === 0) return false;
  if (pos === past.length) draft = typed();
  setInput(past[--pos]);
  return true;
}
function newer() {
  if (pos === past.length) return false;
  setInput(++pos === past.length ? draft : past[pos]);
  return true;
}
/** Enter runs the input, as in a terminal (Tab takes a suggestion), unless a ( [ { is unclosed: then it starts a new line. */
function enter(view) {
  if (busy) return true;
  call("repl_open", typed()).then((open) => (open ? insertNewlineAndIndent(view) : submit()));
  return true;
}

function entry(html) {
  const div = document.createElement("div");
  div.className = "entry";
  div.innerHTML = html;
  prompt.before(div);
  return div;
}
const bottom = () => (log.scrollTop = log.scrollHeight);

/** Greets a new session, saying so if one was lost: a runaway loop or a crash replaces the worker. */
let greeting = null;
const ensureSession = () =>
  (greeting ??= (async () => {
    await warm();
    if (session === generation) return;
    const lost = session !== 0;
    const banner = await call("repl_reset");
    session = generation;
    entry((lost ? `<p class="note">zil restarted, so earlier variables are gone.</p>` : "") + `<pre>${banner}</pre>`);
    bottom();
  })().finally(() => (greeting = null)));

async function submit() {
  const src = typed().replace(/\s+$/, "");
  if (!src.trim()) return;
  if (past.at(-1) !== src) past.push(src);
  past = past.slice(-200);
  pos = past.length;
  draft = "";
  store.set(PAST, JSON.stringify(past));
  setInput("");
  if (src === "clear") return clearLog();
  if (src === "exit" || src === "quit") return reset();
  busy = true;
  // Greet first, so a "session lost" note lands above this input. A load failure shows up from the call below.
  await ensureSession().catch(() => {});
  const div = entry(`<div class="line"><span class="ps">›</span><pre>${highlight(src)}</pre></div>`);
  bottom();
  rstatus.classList.add("busy");
  rstatus.textContent = "Running";
  const t0 = performance.now();
  try {
    const [printed, shown, error] = await call("repl", src);
    div.insertAdjacentHTML(
      "beforeend",
      (printed ? `<pre>${printed}</pre>` : "") + (shown ? `<pre>${shown}</pre>` : "") + (error ? `<div class="error"><pre>${error}</pre></div>` : ""),
    );
    rstatus.textContent = `${Math.max(1, Math.round(performance.now() - t0))} ms`;
  } catch (e) {
    div.insertAdjacentHTML("beforeend", `<div class="error"><pre>${escape(e.message)}</pre></div>`);
    rstatus.textContent = "";
  } finally {
    busy = false;
    rstatus.classList.remove("busy");
    bottom();
    input.focus();
  }
}

function clearLog() {
  for (const e of log.querySelectorAll(".entry")) e.remove();
}
async function reset() {
  clearLog();
  await warm();
  const banner = await call("repl_reset");
  session = generation;
  entry(`<pre>${banner}</pre>`);
  input.focus();
}
$("reset").addEventListener("click", reset);

/** Runs `src` at the REPL prompt, switching to the REPL first. */
async function repl(src) {
  if (busy) return;
  if (mode !== "repl") setMode("repl");
  await ensureSession().catch(() => {});
  setInput(src);
  return submit();
}
/** `help(...)` for a function, unit or module name, as you'd type it. */
const helpFor = (name) => repl(vocab.fns.has(name) ? `help(${name})` : `help(${JSON.stringify(name)})`);

// Help output links its names (`zil:help/upper`) and examples (`zil:run/...`).
document.addEventListener("click", (e) => {
  const a = e.target.closest("a[data-zil]");
  if (!a) return;
  e.preventDefault();
  const [, kind, target] = /^zil:(\w+)\/(.*)$/.exec(a.dataset.zil) ?? [];
  if (kind === "help") helpFor(decodeURIComponent(target));
  else if (kind === "run") repl(decodeURIComponent(target));
});
// Clicking the transcript types at the prompt, unless it was to select text or follow a link.
log.addEventListener("click", (e) => getSelection().isCollapsed && !e.target.closest("a, button") && input.focus());

// ── Modes ────────────────────────────────────────────────────────────────────

const MODE = "zil.mode";
let mode = "editor";
function setMode(m) {
  mode = m;
  document.querySelector(".app").classList.toggle("repl", m === "repl");
  for (const b of document.querySelectorAll("#modes button")) b.setAttribute("aria-selected", String(b.dataset.mode === m));
  store.set(MODE, m);
}
for (const b of document.querySelectorAll("#modes button")) {
  b.addEventListener("click", () => {
    setMode(b.dataset.mode);
    if (mode === "repl") ensureSession().then(() => input.focus());
    else editor.focus();
  });
}

const PASTE = "zil.paste";
let paste = store.get(PASTE) === "replace" ? "replace" : "append";
function setPaste(p) {
  paste = p;
  for (const b of document.querySelectorAll("#paste button")) b.setAttribute("aria-checked", String(b.dataset.paste === p));
  store.set(PASTE, p);
}
setPaste(paste);
for (const b of document.querySelectorAll("#paste button")) b.addEventListener("click", () => setPaste(b.dataset.paste));

// ── The catalog: every example section, searchable; snippets run where you are, whole scripts open in Script mode ──

const catalog = $("catalog"),
  find = $("find");
const OPEN = "zil.open";
let opened = new Set(["a taste"]);
try {
  opened = new Set(JSON.parse(store.get(OPEN)) ?? opened);
} catch {}

/** The help pages, under "start here". They always open in the REPL: only there is help colored and clickable. */
const REFERENCE = [
  ["overview: modules and how to use help", "help()"],
  ["a few dozen one-liners, module by module", 'help("examples")'],
  ["the language at a glance", 'help("syntax")'],
  ["longer recipes combining several modules", 'help("advanced")'],
];

function snippet(label, src, file, inRepl = false) {
  const b = document.createElement("button");
  b.className = "try";
  b.innerHTML = `<span>${escape(label)}</span><code>${highlight(src.split("\n")[0])}</code>`;
  b.title = file ? `Open ${file} in Script mode` : src;
  b.dataset.search = `${label} ${src}`.toLowerCase();
  b.addEventListener("click", async () => {
    if (file) {
      const res = await fetch(`examples/${file}`);
      if (!res.ok) return;
      const text = (await res.text()).replace(/^# Run: .*\n/, "");
      if (mode === "editor" && paste === "append") insert(text);
      else setMode("editor"), load(text);
    } else if (mode === "repl" || inRepl) return repl(src);
    else (paste === "replace" ? load : insert)(src);
    b.setAttribute("aria-current", "true");
  });
  return b;
}

/** Builds the catalog: `[module, title, rows]` sections under a heading per module, then the example files. */
function fill(sections, files) {
  const groups = new Map();
  for (const [module, title, rows] of sections) {
    if (!groups.has(module)) groups.set(module, []);
    groups.get(module).push([title, rows.map(([label, src]) => snippet(label, src))]);
    if (title === "a taste") groups.get(module).push(["reference", REFERENCE.map(([label, src]) => snippet(label, src, null, true))]);
  }
  groups.set("scripts", [["whole scripts", files.map((f) => snippet(f.replace(/\.zil$/, ""), `examples/${f}`, f))]]);
  const nodes = [];
  for (const [module, list] of groups) {
    const h = document.createElement("h3");
    h.textContent = module === "start" ? "start here" : module;
    nodes.push(h);
    for (const [title, buttons] of list) {
      const d = document.createElement("details");
      d.open = opened.has(title);
      d.dataset.title = title;
      d.innerHTML = `<summary><span>${escape(title)}</span><small>${buttons.length}</small></summary>`;
      d.append(...buttons);
      d.addEventListener("toggle", () => {
        if (find.value) return; // a search opens sections; that's not the reader's choice
        d.open ? opened.add(title) : opened.delete(title);
        store.set(OPEN, JSON.stringify([...opened]));
      });
      nodes.push(d);
    }
  }
  catalog.replaceChildren(...nodes);
}

/** Shows only snippets matching every word of the search, opening their sections; an empty search restores what was open. */
function filter() {
  const words = find.value.toLowerCase().split(/\s+/).filter(Boolean);
  let hits = 0;
  for (const d of catalog.querySelectorAll("details")) {
    const title = d.dataset.title.toLowerCase();
    let any = false;
    for (const b of d.querySelectorAll(".try")) {
      const hit = words.every((w) => b.dataset.search.includes(w) || title.includes(w));
      b.hidden = !hit;
      any ||= hit;
    }
    d.hidden = !any;
    d.open = words.length ? any : opened.has(d.dataset.title);
    hits += any;
  }
  for (const h of catalog.querySelectorAll("h3")) {
    let n = h.nextElementSibling,
      any = false;
    for (; n && n.tagName === "DETAILS"; n = n.nextElementSibling) any ||= !n.hidden;
    h.hidden = !any;
  }
  $("nohits").hidden = !words.length || hits > 0;
}
find.addEventListener("input", filter);
find.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    find.value = "";
    filter();
  } else if (e.key === "Enter") catalog.querySelector(".try:not([hidden])")?.click();
});
// `/` jumps to the search, unless you're typing somewhere.
document.addEventListener("keydown", (e) => {
  if (e.key !== "/" || e.ctrlKey || e.metaKey || e.target.closest("input, textarea, .cm-editor")) return;
  e.preventDefault();
  find.focus();
});

// ── The tour ─────────────────────────────────────────────────────────────────

const tour = $("tour");
// The steps from `guide.rs`, flattened: {chapter, text, code}. The last one opens Script mode.
const steps = call("tour").then((json) => JSON.parse(json).flatMap(([chapter, rows], c) => rows.map(([text, code]) => ({ c, chapter, text, code }))));
const chapters = steps.then((all) => [...new Set(all.map((s) => s.chapter))]);
const STEP = "zil.tour.step";
let step = 0,
  ran = false;
async function showStep() {
  const all = await steps;
  const s = all[step];
  const inChapter = all.filter((t) => t.c === s.c);
  const options = (await chapters).map((name, i) => `<option value="${i}"${i === s.c ? " selected" : ""}>${escape(name)}</option>`);
  const prose = escape(s.text).replace(/`([^`]+)`/g, "<code>$1</code>");
  const last = step === all.length - 1;
  store.set(STEP, step);
  tour.innerHTML = `
    <header><select data-act="chapter" aria-label="Tour chapter">${options.join("")}</select><span>· ${inChapter.indexOf(s) + 1} of ${inChapter.length}</span><button class="x" data-act="close" aria-label="Close the tour">×</button></header>
    <p>${prose}</p>
    ${s.code ? `<pre>${highlight(s.code)}</pre>` : ""}
    <footer>
      ${step ? `<button class="btn" data-act="back">Back</button>` : ""}
      <span class="spacer"></span>
      ${
        last
          ? `<button class="btn primary" data-act="script">Open Script mode</button>`
          : ran || !s.code
            ? `<button class="btn primary" data-act="next">Next</button>`
            : `<button class="btn" data-act="next">Skip</button><button class="btn primary" data-act="run">Run it</button>`
      }
    </footer>`;
}
async function go(to) {
  step = to;
  ran = false;
  await showStep();
  tour.querySelector("[data-act=run], [data-act=next], [data-act=script]")?.focus();
}
tour.addEventListener("click", async (e) => {
  const act = e.target.closest("button[data-act]")?.dataset.act;
  if (act === "run") {
    ran = true;
    showStep();
    await repl((await steps)[step].code);
  } else if (act === "next" || act === "back") go(step + (act === "next" ? 1 : -1));
  else if (act === "script") {
    endTour();
    store.set(STEP, 0);
    setMode("editor");
    load(START);
  } else if (act === "close") endTour();
});
tour.addEventListener("change", async (e) => {
  if (e.target.dataset.act === "chapter") go((await steps).findIndex((s) => s.c === +e.target.value));
});
// Opens where the reader left off.
async function startTour() {
  const all = await steps;
  tour.hidden = false;
  go(Math.min(+store.get(STEP) || 0, all.length - 1));
}
function endTour() {
  tour.hidden = true;
  store.set("zil.toured", "1");
}
$("tour-start").addEventListener("click", startTour);

// ── Start: shared code, else what was here last time, else the welcome script ──

const shared = location.hash.match(/^#code=([\w-]*)/);
editor.dispatch({ changes: { from: 0, insert: shared ? decode(shared[1]) : (store.get(SAVED) ?? START) } });
// Mode: #repl links open the REPL, shared code opens the editor, else whichever was used last.
setMode(location.hash !== "#repl" && (shared || store.get(MODE) === "editor") ? "editor" : "repl");
if (!shared && !store.get("zil.toured")) startTour();

warm()
  .then(async () => {
    learn(await call("vocab"));
    relearn(editor);
    relearn(input);
    const names = await fetch("examples/index.txt")
      .then((r) => (r.ok ? r.text() : ""))
      .catch(() => "");
    fill(JSON.parse(await call("examples")), names.split("\n").filter(Boolean));
    run();
    if (mode === "repl") ensureSession().then(() => tour.hidden && input.focus());
  })
  .catch((e) => (out.innerHTML = `<div class="error"><pre>${escape(e.message)}</pre></div>`));
