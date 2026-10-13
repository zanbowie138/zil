<!-- The tour: `guide.rs`'s TOUR a step at a time, each step's code run in the REPL. Opens where the reader left off. -->
<script>
  import { Select } from "bits-ui";
  import { onMount } from "svelte";
  import { escape, highlight } from "../lib/client.js";
  import { store } from "./store.js";

  /** `steps` resolves to `[{c, chapter, text, code}]`; the last one opens Script mode. */
  let { steps, onRun, onScript, onClose } = $props();
  const STEP = "zil.tour.step";

  let all = $state([]);
  let step = $state(0),
    ran = $state(false);
  let el = $state();
  onMount(() =>
    steps.then((s) => {
      all = s;
      go(Math.min(+store.get(STEP) || 0, s.length - 1));
    }),
  );

  const s = $derived(all[step]);
  const chapters = $derived([...new Set(all.map((t) => t.chapter))]);
  const inChapter = $derived(s ? all.filter((t) => t.c === s.c) : []);
  const last = $derived(step === all.length - 1);

  function go(to) {
    step = to;
    ran = false;
    store.set(STEP, to);
    // After the buttons re-render, focus the one that moves on.
    requestAnimationFrame(() => el?.querySelector("[data-next]")?.focus());
  }
  async function run() {
    ran = true;
    await onRun(s.code);
  }
  function script() {
    store.set(STEP, 0);
    onScript();
  }
</script>

{#if s}
  <div class="tour" aria-live="polite" bind:this={el}>
    <header>
      <Select.Root type="single" value={String(s.c)} onValueChange={(v) => go(all.findIndex((t) => t.c === +v))} items={chapters.map((name, i) => ({ value: String(i), label: name }))}>
        <Select.Trigger class="chapter" aria-label="Tour chapter">{s.chapter}</Select.Trigger>
        <Select.Portal>
          <Select.Content class="menu" sideOffset={4}>
            <Select.Viewport>
              {#each chapters as name, i}
                <Select.Item class="menu-item" value={String(i)} label={name}>{name}</Select.Item>
              {/each}
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
      <span>· {inChapter.indexOf(s) + 1} of {inChapter.length}</span>
      <button class="x" onclick={onClose} aria-label="Close the tour">×</button>
    </header>
    <p>{@html escape(s.text).replace(/`([^`]+)`/g, "<code>$1</code>")}</p>
    {#if s.code}<pre>{@html highlight(s.code)}</pre>{/if}
    <footer>
      {#if step}<button class="btn" onclick={() => go(step - 1)}>Back</button>{/if}
      <span class="spacer"></span>
      {#if last}
        <button class="btn primary" data-next onclick={script}>Open Script mode</button>
      {:else if ran || !s.code}
        <button class="btn primary" data-next onclick={() => go(step + 1)}>Next</button>
      {:else}
        <button class="btn" onclick={() => go(step + 1)}>Skip</button>
        <button class="btn primary" data-next onclick={run}>Run it</button>
      {/if}
    </footer>
  </div>
{/if}
