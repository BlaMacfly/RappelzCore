// Importe le schéma SQL d'AzerothCore (data/sql/base) dans RappelzCore.
//
// Usage : node import_acore_schema.js <azerothcore/data/sql/base> <RappelzCore/sql/base>
//
// - db_auth et db_characters : copiés tels quels (structure + petites données de base),
//   sauf les données de `updates` qui référencent les migrations d'AzerothCore.
// - db_world : structure uniquement (DROP/CREATE), les INSERT sont retirés ; les données
//   world de RappelzCore viennent du client Rappelz via `rz-dbimport`.
const fs = require("fs");
const path = require("path");

const STRUCT_ONLY = new Set(["db_world"]);
const SKIP_DATA_TABLES = new Set(["updates"]);

function stripInserts(text) {
  const out = [];
  let inInsert = false;
  for (const line of text.split(/(?<=\n)/)) {
    if (inInsert) {
      if (line.trimEnd().endsWith(";")) inInsert = false;
      continue;
    }
    if (/^INSERT INTO/i.test(line)) {
      if (!line.trimEnd().endsWith(";")) inInsert = true;
      continue;
    }
    out.push(line);
  }
  return out.join("");
}

const [src, dst] = process.argv.slice(2);
for (const db of ["db_auth", "db_characters", "db_world"]) {
  const sdir = path.join(src, db);
  const ddir = path.join(dst, db);
  fs.mkdirSync(ddir, { recursive: true });
  let n = 0;
  for (const name of fs.readdirSync(sdir).sort()) {
    if (!name.endsWith(".sql")) continue;
    let text = fs.readFileSync(path.join(sdir, name), "utf8").replace(/\r\n/g, "\n");
    if (STRUCT_ONLY.has(db) || SKIP_DATA_TABLES.has(name.slice(0, -4))) text = stripInserts(text);
    fs.writeFileSync(path.join(ddir, name), text);
    n++;
  }
  console.log(`${db}: ${n} tables`);
}
