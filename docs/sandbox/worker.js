// Runs zil off the page's thread, so a runaway loop can be killed with worker.terminate().
// Messages in: {id, fn: "run" | "notebook" | "examples", src}. Out: {ready} once loaded, then {id, result} or {id, crash}.
import init, * as zil from "./pkg/zil.js";

const ready = init().then(() => postMessage({ ready: true }));

onmessage = async ({ data: { id, fn, src } }) => {
  await ready;
  try {
    postMessage({ id, result: zil[fn](src) });
  } catch (e) {
    // A Rust panic traps the whole instance; the page throws this worker away.
    postMessage({ id, crash: String(e) });
  }
};
