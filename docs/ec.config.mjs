// Code blocks: a TextMate grammar for zil, and light and dark themes in the site's palette (theme.css's --k, --s, ... tokens).
// The grammar can't know which names are units; it guesses from position (`42 km`, `to mi`). `lib/client.js` highlights edited blocks.
import { defineEcConfig } from "@astrojs/starlight/expressive-code";

const el = (tagName, className, children = []) => ({ type: "element", tagName, properties: className ? { className: className.split(" ") } : {}, children });
const txt = (value) => ({ type: "text", value });

/** zil blocks as an editor: line numbers, the `# → ` result lines `zil --docs` writes shown beside their line
 * (or under it, if several lines), errors marked in the gutter, and a status bar. `lib/runnable.js` reads this back for CodeMirror. */
function results() {
  const RESULT = /^\s*# →(?: |$)/;
  const found = new WeakMap();
  const bad = (t) => t.startsWith("error: ");
  return {
    name: "zil-results",
    hooks: {
      preprocessCode({ codeBlock, addGutterElement }) {
        if (codeBlock.language !== "zil") return;
        const rows = new Map(),
          drop = [];
        let at = -1;
        codeBlock.getLines().forEach((line, i) => {
          if (!RESULT.test(line.text)) return void (at = i - drop.length);
          drop.push(i);
          const t = line.text.replace(RESULT, "");
          if (at >= 0) rows.set(at, rows.has(at) ? `${rows.get(at)}\n${t}` : t);
        });
        if (drop.length) codeBlock.deleteLines(drop);
        found.set(codeBlock, rows);
        addGutterElement({
          renderPhase: "earlier",
          renderLine: ({ lineIndex }) => el("div", "ln", [txt(String(lineIndex + 1))]),
          renderPlaceholder: () => el("div", "ln"),
        });
      },
      postprocessRenderedLine({ codeBlock, lineIndex, renderData }) {
        const t = found.get(codeBlock)?.get(lineIndex);
        if (t === undefined) return;
        if (bad(t)) renderData.lineAst.children[0].children[0].properties.className.push("zil-err");
        if (t.includes("\n")) return;
        const span = el("span", bad(t) ? "zil-inline bad" : "zil-inline", [txt(t)]);
        span.properties.title = t;
        renderData.lineAst.children.at(-1).children.push(span);
      },
      postprocessRenderedBlock({ codeBlock, renderData, renderEmptyLine }) {
        const rows = found.get(codeBlock);
        if (!rows) return;
        const find = (n, tag) => (n.tagName === tag ? n : n.children?.map((c) => find(c, tag)).find(Boolean));
        const code = find(renderData.blockAst, "code");
        const lines = code.children.filter((n) => n.tagName === "div");
        code.children = lines.flatMap((line, i) => {
          const t = rows.get(i);
          if (!t?.includes("\n")) return [line];
          const out = renderEmptyLine();
          out.lineAst.properties.className.push("zil-out-line");
          out.codeWrapper.children = [el("div", bad(t) ? "zil-out bad" : "zil-out", [txt(t)])];
          return [line, out.lineAst];
        });
        renderData.blockAst.children.push(el("div", "zil-status", [el("span", "", [txt("Example output")]), el("span", "", [txt("Ctrl+Enter to run")])]));
      },
    },
  };
}

const KEYWORDS = "fn|if|else|while|for|in|to|of|return|break|continue|true|false|nil";
const NUMBER = String.raw`\d+#[0-9a-zA-Z_]+|\d+[dhms](?:\d+[dhms])+\b|\d{1,2}:\d{2}(?::\d{2})?(?:[ap]m)?|\d{1,2}[ap]m\b|0[xbo][0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?`;
const IDENT = String.raw`[A-Za-z_ΩµμΩ°][\wΩµμΩ°]*`;

const zil = {
  name: "zil",
  scopeName: "source.zil",
  patterns: [
    { match: String.raw`^\s*# → .*$`, name: "comment.result.zil" },
    // A unit right after a number.
    { match: String.raw`(?<![\w.])(${NUMBER})\s?(?!(?:${KEYWORDS})\b)(${IDENT})(?!\s*\()`, captures: { 1: { name: "constant.numeric.zil" }, 2: { name: "support.type.unit.zil" } } },
    { match: String.raw`(?<![\w.])(?:${NUMBER})`, name: "constant.numeric.zil" },
    { match: "#.*$", name: "comment.line.zil" },
    { begin: 'r?"', end: '"', name: "string.quoted.zil", patterns: [{ match: String.raw`\\.`, name: "constant.character.escape.zil" }] },
    // A conversion target: `to mi`, but not `to hex(8)`.
    { match: String.raw`\b(to)\s+(?!(?:${KEYWORDS})\b)(${IDENT})\b(?!\s*\()`, captures: { 1: { name: "keyword.control.zil" }, 2: { name: "support.type.unit.zil" } } },
    { match: `\\b(?:${KEYWORDS})\\b`, name: "keyword.control.zil" },
    { match: String.raw`(\|>)\s*(${IDENT})`, captures: { 1: { name: "keyword.operator.zil" }, 2: { name: "entity.name.function.zil" } } },
    { match: String.raw`${IDENT}(?=\()`, name: "entity.name.function.zil" },
    { match: String.raw`(?<=\.)${IDENT}`, name: "entity.name.function.zil" },
    { match: String.raw`\b(?:pi|tau|phi|inf|nan)\b`, name: "constant.language.zil" },
    { match: String.raw`\|>|±|\+-|\.\.=?|[$€£]`, name: "keyword.operator.zil" },
  ],
};

/** A theme from theme.css's palette for one color scheme. */
const theme = (type, c) => ({
  name: `zil-${type}`,
  type,
  colors: { "editor.background": c.panel, "editor.foreground": c.ink, "editor.selectionBackground": c.select, "editorLineNumber.foreground": c.faint },
  tokenColors: [
    { scope: ["keyword", "keyword.operator"], settings: { foreground: c.k } },
    { scope: ["string", "constant.character.escape"], settings: { foreground: c.s } },
    { scope: ["constant.numeric", "constant.language"], settings: { foreground: c.n } },
    { scope: ["entity.name.function"], settings: { foreground: c.f } },
    { scope: ["support.type.unit"], settings: { foreground: c.u } },
    { scope: ["comment"], settings: { foreground: c.c, fontStyle: "italic" } },
    { scope: ["comment.result"], settings: { foreground: c.muted, fontStyle: "" } },
  ],
});

export default defineEcConfig({
  shiki: { langs: [zil] },
  plugins: [results()],
  themes: [
    theme("dark", { panel: "#1a1a18", ink: "#ebe9e3", muted: "#9a958b", faint: "#67635b", select: "#ff7b3f38", k: "#ff9a62", s: "#a5d46a", n: "#5ccfd6", f: "#8fb0ff", u: "#d7a2ef", c: "#6f6b62" }),
    theme("light", { panel: "#fffefb", ink: "#1d1c1a", muted: "#6e6a62", faint: "#a6a196", select: "#c8440f38", k: "#b3420e", s: "#3f7a12", n: "#0f6e76", f: "#2849b8", u: "#8a3aa8", c: "#9b968b" }),
  ],
  styleOverrides: {
    borderRadius: "0.6rem",
    borderColor: "var(--line)",
    codeFontFamily: "var(--term)",
    codeFontSize: "0.85rem",
    uiFontFamily: "var(--sans)",
    frames: { shadowColor: "transparent", editorTabBarBackground: "var(--sunk)", terminalTitlebarBackground: "var(--sunk)" },
  },
});
