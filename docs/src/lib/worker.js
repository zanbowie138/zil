// Runs zil off the page's thread, so a runaway loop can be killed with worker.terminate().
// Messages in: {id, fn, src}, fn naming a `wasm.rs` export. Out: {ready} or {fail} once loaded, then {id, result} or {id, crash}.
import init, * as zil from "./pkg/zil.js";

// zil's network calls (currency rates, fetch, translate) block, which only a worker's synchronous XHR can do.
globalThis.zilGet = (url, query) => {
  const u = new URL(url);
  for (const [k, v] of JSON.parse(query)) u.searchParams.append(k, v);
  const x = new XMLHttpRequest();
  try {
    x.open("GET", u, false);
    x.send();
  } catch {
    throw `couldn't reach ${u.host}`;
  }
  if (x.status !== 200) throw `${u.host} answered ${x.status}`;
  return x.responseText;
};

// A failed init is a promise rejection, which the page's worker.onerror never sees, so say so.
const ready = init().then(
  () => postMessage({ ready: true }),
  (e) => postMessage({ fail: String(e) }),
);

onmessage = async ({ data: { id, fn, src } }) => {
  await ready;
  try {
    postMessage({ id, result: zil[fn](src) });
  } catch (e) {
    // A Rust panic traps the whole instance; the page throws this worker away.
    postMessage({ id, crash: String(e) });
  }
};
