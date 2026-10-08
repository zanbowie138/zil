// Makes the book's zil code blocks editable and runnable, on the sandbox's wasm build (../sandbox, from `pnpm build`).
// Running a block replaces its `# → ` result lines with fresh ones, so edits show their results in place.
(async () => {
  const here = document.currentScript.src;
  const sandbox = new URL("sandbox/", here);
  const zil = await import(new URL("client.js", sandbox));

  // A "Sandbox" link in the top bar, next to mdBook's own buttons.
  const right = document.querySelector(".right-buttons");
  if (right) {
    const a = document.createElement("a");
    a.href = sandbox;
    a.className = "zil-sandbox-link";
    a.textContent = "Sandbox";
    right.prepend(a);
  }

  const RESULT = /^\s*# → /;
  /** The block without its result lines: what the reader wrote. */
  const source = (text) => text.replace(/\n$/, "").split("\n").filter((l) => !RESULT.test(l)).join("\n");
  /** `src` with each `[line, text]` from `notebook` written as `# → ` lines after its line. */
  function merge(src, rows) {
    const after = new Map(rows);
    return src
      .split("\n")
      .flatMap((line, i) => (after.has(i) ? [line, ...after.get(i).split("\n").map((r) => `# → ${r}`)] : [line]))
      .join("\n");
  }

  for (const code of document.querySelectorAll("pre > code.language-zil")) {
    const pre = code.parentElement;
    code.innerHTML = zil.highlight(code.textContent);
    code.contentEditable = "plaintext-only";
    code.spellcheck = false;

    const run = document.createElement("button");
    run.className = "zil-run";
    run.textContent = "Run";
    run.title = "Run this example (Ctrl+Enter while editing). Edit it first if you like.";
    run.addEventListener("pointerenter", zil.warm, { once: true });
    pre.append(run);

    let busy = false;
    async function go() {
      if (busy) return;
      busy = true;
      run.textContent = "Running…";
      const src = source(code.textContent);
      let rows;
      try {
        rows = JSON.parse(await zil.call("notebook", src));
      } catch (e) {
        rows = [[src.split("\n").length - 1, e.message]];
      }
      code.innerHTML = zil.highlight(merge(src, rows));
      run.textContent = "Run";
      busy = false;
    }
    run.addEventListener("click", go);
    code.addEventListener("keydown", (e) => {
      // Keep mdBook's arrow-key page turns and shortcuts out of the editor.
      e.stopPropagation();
      if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        go();
      }
    });

    // In mdBook's hover row: open this block in the sandbox.
    const open = document.createElement("button");
    open.className = "zil-open";
    open.title = "Open in the sandbox";
    open.setAttribute("aria-label", open.title);
    open.addEventListener("click", () => window.open(`${sandbox}#code=${zil.encode(source(code.textContent))}`, "_blank"));
    let buttons = pre.querySelector(".buttons");
    if (!buttons) {
      buttons = document.createElement("div");
      buttons.className = "buttons";
      pre.prepend(buttons);
    }
    buttons.prepend(open);
  }
})();
