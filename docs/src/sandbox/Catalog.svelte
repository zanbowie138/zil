<!-- The catalog: every example section, searchable; snippets run where you are, whole scripts open in Script mode. -->
<script>
  import { Accordion, ToggleGroup } from "bits-ui";
  import { highlight } from "../lib/client.js";
  import { store } from "./store.js";

  /** `sections` is the wasm's `examples()`, `[[module, title, [[label, code], ...]], ...]`, or null while it loads. */
  let { sections, current, paste = $bindable(), onPick, onTour, onPaste, children } = $props();

  /** The whole scripts in examples/, loaded when picked. */
  const scripts = import.meta.glob("../../../examples/*.zil", { query: "?raw", import: "default" });
  /** The help pages, under "start here". They always open in the REPL: only there is help colored and clickable. */
  const REFERENCE = [
    ["overview: modules and how to use help", "help()"],
    ["a few dozen one-liners, module by module", 'help("examples")'],
    ["the language at a glance", 'help("syntax")'],
    ["longer recipes combining several modules", 'help("advanced")'],
  ];
  const OPEN = "zil.open";
  /** The sections the reader opened, by title. */
  let opened = $state(saved());
  function saved() {
    try {
      return JSON.parse(store.get(OPEN)) ?? ["a taste"];
    } catch {
      return ["a taste"];
    }
  }
  let query = $state("");

  const item = (id, label, src, extra = {}) => ({ id, label, src, search: `${label} ${src}`.toLowerCase(), ...extra });
  /** `[module, [{title, items}]]` with a heading per module, then the example scripts. */
  const groups = $derived.by(() => {
    if (!sections) return [];
    const out = new Map();
    for (const [module, title, rows] of sections) {
      if (!out.has(module)) out.set(module, []);
      out.get(module).push({ title, items: rows.map(([label, src], i) => item(`${title}/${i}`, label, src)) });
      if (title === "a taste") out.get(module).push({ title: "reference", items: REFERENCE.map(([label, src], i) => item(`reference/${i}`, label, src, { inRepl: true })) });
    }
    const files = Object.keys(scripts).map((path) => path.split("/").pop());
    out.set("scripts", [{ title: "whole scripts", items: files.map((f) => item(`scripts/${f}`, f.replace(/\.zil$/, ""), `examples/${f}`, { file: f })) }]);
    return [...out];
  });

  const words = $derived(query.toLowerCase().split(/\s+/).filter(Boolean));
  /** Snippets matching every word of the search, by section. */
  const hits = $derived.by(() => {
    const hits = new Map();
    for (const [, list] of groups)
      for (const s of list) {
        const title = s.title.toLowerCase();
        hits.set(s, words.length ? s.items.filter((it) => words.every((w) => it.search.includes(w) || title.includes(w))) : s.items);
      }
    return hits;
  });
  /** A search opens the sections it matches; an empty one restores what the reader had open. */
  const open = $derived(words.length ? [...hits].filter(([, h]) => h.length).map(([s]) => s.title) : opened);
  const none = $derived(words.length > 0 && [...hits.values()].every((h) => !h.length));

  function toggled(value) {
    if (words.length) return; // a search opens sections; that's not the reader's choice
    opened = value;
    store.set(OPEN, JSON.stringify(value));
  }

  async function choose(it) {
    if (!it.file) return onPick(it);
    const text = await scripts[`../../../examples/${it.file}`]();
    onPick({ ...it, src: text.replace(/^# Run: .*\n/, "") });
  }

  function keydown(e) {
    if (e.key === "Escape") query = "";
    else if (e.key === "Enter") {
      const first = [...hits.values()].find((h) => h.length)?.[0];
      if (first) choose(first);
    }
  }
</script>

<div class="search">
  <input type="search" bind:value={query} onkeydown={keydown} placeholder="Search examples" aria-label="Search examples" />
  <button class="btn" onclick={onTour} title="A five-minute walk through the language">Tour</button>
</div>
<div class="paste editor-only">
  <span id="paste-label">Examples</span>
  <ToggleGroup.Root class="modes" type="single" value={paste} onValueChange={(p) => p && ((paste = p), onPaste(p))} aria-labelledby="paste-label">
    <ToggleGroup.Item value="append" title="Add the example after the cursor">Append</ToggleGroup.Item>
    <ToggleGroup.Item value="replace" title="Replace the whole script with the example">Replace</ToggleGroup.Item>
  </ToggleGroup.Root>
</div>
{@render children?.()}
<div class="scroll">
  {#if !sections}
    <p class="empty">Loading examples…</p>
  {:else}
    <Accordion.Root type="multiple" value={open} onValueChange={toggled}>
      {#each groups as [module, list]}
        {#if list.some((s) => hits.get(s).length)}
          <h3>{module === "start" ? "start here" : module}</h3>
        {/if}
        {#each list as s}
          {#if hits.get(s).length}
            <Accordion.Item value={s.title} class="section">
              <Accordion.Header level={4} class="section-head">
                <Accordion.Trigger class="summary"><span>{s.title}</span><small>{s.items.length}</small></Accordion.Trigger>
              </Accordion.Header>
              <Accordion.Content>
                {#each hits.get(s) as it (it.id)}
                  <button class="try" aria-current={current === it.id || undefined} title={it.file ? `Open ${it.file} in Script mode` : it.src} onclick={() => choose(it)}>
                    <span>{it.label}</span><code>{@html highlight(it.src.split("\n")[0])}</code>
                  </button>
                {/each}
              </Accordion.Content>
            </Accordion.Item>
          {/if}
        {/each}
      {/each}
    </Accordion.Root>
  {/if}
  {#if none}
    <p class="empty">No examples match. Try <code>help("…")</code> in the REPL to search every function.</p>
  {/if}
  <p class="credits">
    City and country data from <a href="https://www.geonames.org/">GeoNames</a> (<a href="https://creativecommons.org/licenses/by/4.0/">CC BY 4.0</a>). Editor by
    <a href="https://codemirror.net/">CodeMirror</a> (MIT).
  </p>
</div>
