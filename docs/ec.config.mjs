// Code blocks: a TextMate grammar for zil, and light and dark themes in the site's palette (theme.css's --k, --s, ... tokens).
// The grammar can't know which names are units; it guesses from position (`42 km`, `to mi`). `lib/client.js` highlights edited blocks.
import { defineEcConfig } from "@astrojs/starlight/expressive-code";

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
