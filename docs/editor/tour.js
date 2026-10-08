// The first-visit tour: each step explains one idea and runs one line in the REPL, so later steps can use earlier results.
// `text` is HTML. A step with `script` instead of `code` opens Script mode on that script.

export const TOUR = [
  {
    text: "zil is a calculator first. Numbers carry units, and <code>to</code> converts them.",
    code: "42 km / 65 mph to min",
  },
  {
    text: "Every result is saved: <code>_</code> is the last one, and <code>_1</code>, <code>_2</code>… are numbered ones.",
    code: "_ * 2",
  },
  {
    text: "Decimals are exact, and integers never overflow.",
    code: "0.1 + 0.2 == 0.3",
  },
  {
    text: "Dates know time zones, and city names work as time zones.",
    code: 'date("2026-12-25 18:30") to "Tokyo"',
  },
  {
    text: "<code>x.f(y)</code> is <code>f(x, y)</code>, and calls without arguments need no parens. Type <code>\"hi\".</code> at the prompt to see every method, or hover a name for its docs.",
    code: '"the quick brown fox".split.map(|w| w.capitalize).join(" ")',
  },
  {
    text: "<code>|x| ...</code> is a function and <code>|&gt;</code> pipes a value into one.",
    code: "scores = [4, 8, 15, 16, 23, 42]\nscores.filter(|x| x > 10) |> avg",
  },
  {
    text: "Help is built in and every example in it runs live. Click a name or an example in the output to follow it.",
    code: "help(sparkline)",
  },
  {
    text: "Search help by topic when you don't know a function's name.",
    code: 'help("sorting")',
  },
  {
    text: "For longer programs, use Script mode: every line shows its value as you type, and Share copies a link to your code.",
    script: true,
  },
];
