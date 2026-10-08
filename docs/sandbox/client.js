// The page side of zil in the browser, shared by the sandbox and the book's run buttons.

/** A run longer than this is a runaway loop: its worker is killed and replaced. */
export const LIMIT_MS = 10_000;

let worker, ready, nextId = 0;
const pending = new Map();
/** Counts workers started: a new one means anything kept in the old one, like the REPL's variables, is gone. */
export let generation = 0;

function spawn() {
  generation++;
  worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  ready = new Promise((resolve, reject) => {
    worker.onmessage = ({ data }) => {
      if (data.ready) return resolve();
      const p = pending.get(data.id);
      if (!p) return;
      pending.delete(data.id);
      clearTimeout(p.timer);
      if ("crash" in data) {
        kill(`zil crashed: ${data.crash}`);
        p.reject(new Error(`zil crashed: ${data.crash}`));
      } else p.resolve(data.result);
    };
    worker.onerror = (e) => {
      const why = `couldn't load zil: ${e.message || "network error"}`;
      kill(why);
      reject(new Error(why));
    };
  });
}

/** Throws away the worker and fails whatever it was running. */
function kill(reason) {
  worker?.terminate();
  worker = null;
  for (const p of pending.values()) {
    clearTimeout(p.timer);
    p.reject(new Error(reason));
  }
  pending.clear();
}

/** Starts loading zil now, so the first run doesn't wait for the download. */
export function warm() {
  if (!worker) spawn();
  return ready;
}

/** Calls a `wasm.rs` export in the worker, like "run", "notebook" or "repl". */
export async function call(fn, src = "") {
  await warm();
  const id = nextId++;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => kill(`stopped after ${LIMIT_MS / 1000} s; is there an endless loop?`), LIMIT_MS);
    pending.set(id, { resolve, reject, timer });
    worker.postMessage({ id, fn, src });
  });
}

/** Stops a run in progress. */
export const stop = () => kill("stopped");

const KEYWORDS = "fn|if|else|while|for|in|to|of|return|break|continue|true|false|nil|unit";
const TOKEN = new RegExp(
  [
    /(?<result>^[ \t]*# → .*$)/,
    /(?<comment>#.*$)/,
    /(?<string>r?"(?:\\.|[^"\\])*"?)/,
    /(?<number>\b0[xbo][0-9a-fA-F_]+|\b\d[\d_]*(?:\.\d+)?(?:[eE][+-]?\d+)?)/,
    new RegExp(`(?<keyword>\\b(?:${KEYWORDS})\\b)`),
    /(?<call>\b[A-Za-z_]\w*(?=\())/,
  ]
    .map((r) => r.source)
    .join("|"),
  "gm",
);
const CLASS = { result: "zil-result", comment: "hljs-comment", string: "hljs-string", number: "hljs-number", keyword: "hljs-keyword", call: "hljs-title" };

export const escape = (s) => s.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[c]);

/** zil source as HTML, using highlight.js class names so mdBook's themes color it too. */
export function highlight(src) {
  let out = "",
    last = 0;
  for (const m of src.matchAll(TOKEN)) {
    const kind = Object.keys(m.groups).find((k) => m.groups[k] !== undefined);
    out += escape(src.slice(last, m.index)) + `<span class="${CLASS[kind]}">${escape(m[0])}</span>`;
    last = m.index + m[0].length;
  }
  return out + escape(src.slice(last));
}

/** Sandbox links carry the code in the URL hash as base64url UTF-8. */
export function encode(src) {
  const bin = String.fromCharCode(...new TextEncoder().encode(src));
  return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export function decode(s) {
  const bin = atob(s.replace(/-/g, "+").replace(/_/g, "/"));
  return new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
}
