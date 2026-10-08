// File copying for the package.json scripts, in Node so they run the same under cmd and sh.
//   node docs/stage.mjs examples   examples/*.zil into the sandbox, plus index.txt listing them for its Examples menu
//   node docs/stage.mjs site       the site into site/: the sandbox at the root, the built book at docs/
import { cpSync, mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";

const what = process.argv[2];
if (what === "examples") {
  const names = readdirSync("examples").filter((f) => f.endsWith(".zil"));
  mkdirSync("docs/sandbox/examples", { recursive: true });
  for (const f of names) cpSync(`examples/${f}`, `docs/sandbox/examples/${f}`);
  writeFileSync("docs/sandbox/examples/index.txt", names.join("\n") + "\n");
} else if (what === "site") {
  rmSync("site", { recursive: true, force: true });
  cpSync("docs/sandbox", "site", { recursive: true });
  cpSync("docs/book/book", "site/docs", { recursive: true });
} else {
  console.error("usage: node docs/stage.mjs examples|site");
  process.exit(2);
}
