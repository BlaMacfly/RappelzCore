// Génère shared/rz-rappelz/src/rdb_schemas.rs à partir des schémas validés de
// docs/re/tools/rdb_dump.js (source de vérité de la rétro-ingénierie).
// Usage : node tools/gen_rdb_schemas.js
const fs = require("fs");
const path = require("path");
const root = path.join(__dirname, "..");
const src = fs.readFileSync(path.join(root, "docs/re/tools/rdb_dump.js"), "utf8");
const start = src.indexOf("const SCHEMAS = {");
const end = src.indexOf("\n};", start);
const SCHEMAS = eval("(" + src.slice(start + "const SCHEMAS = ".length, end + 2) + ")");
const ty = (t) => {
  if (t[0] === "c") return `FieldType::Char(${parseInt(t.slice(1), 10)})`;
  return { bit: "FieldType::Bit", i8: "FieldType::I8", i16: "FieldType::I16", i32: "FieldType::I32",
    f32: "FieldType::F32", f64: "FieldType::F64", dec2: "FieldType::Dec2" }[t] || (() => { throw new Error(t); })();
};
let out = "// Fichier généré par tools/gen_rdb_schemas.js — ne pas modifier à la main.\n";
out += "// Source : docs/re/tools/rdb_dump.js (schémas validés sur le client NoLimits).\n\n";
out += "use crate::rdb::FieldType;\n\n";
out += "pub static SCHEMAS: &[(&str, &[(&str, FieldType)])] = &[\n";
for (const [table, fields] of Object.entries(SCHEMAS)) {
  out += `    (\n        "${table}",\n        &[\n`;
  for (const [name, t] of fields) out += `            ("${name}", ${ty(t)}),\n`;
  out += "        ],\n    ),\n";
}
out += "];\n";
fs.writeFileSync(path.join(root, "shared/rz-rappelz/src/rdb_schemas.rs"), out);
console.log(`${Object.keys(SCHEMAS).length} schémas, ${Object.values(SCHEMAS).reduce((a, f) => a + f.length, 0)} champs`);
