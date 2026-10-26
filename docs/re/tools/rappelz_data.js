#!/usr/bin/env node
// Lecteur de l'archive de ressources Rappelz (data.000 = index, data.001..data.008 = contenu).
// Reconstruit par RE de SFrame.exe (NoLimits, sha256 27f42fc8...), cf. docs/re/data_archive.md.
// LECTURE SEULE : n'ouvre jamais les fichiers du client en écriture.
//
// Usage :
//   node rappelz_data.js <dossier_client> selftest [dossier_sortie]
//   node rappelz_data.js <dossier_client> stats
//   node rappelz_data.js <dossier_client> list [regex]
//   node rappelz_data.js <dossier_client> extract <regex> <dossier_sortie>
'use strict';
const fs = require('fs');
const path = require('path');

const hex = (s) => Buffer.from(s.replace(/\s+/g, ''), 'hex');

// SFrame.exe 0x00dfb1a8 (copie identique en 0x00dfb0a8, utilisée par le chemin XOR 32 bits)
const XOR_KEY = hex(`
77e85eecb74ec1874fe6f53c1fb315436a4930a6bf53a8355be59e0e41ec22b8
d480a48cce65131d4b085a6abb6fad25b8ddcc773074ac8c5a4a9a9b36bc530a
3cf8960b5daa28a9b282136ef1c193a99e5f20cfd4cc5b2e16f5c94cb21c57ee
14edf97297221b4aa42eb896ef4b3f8eab605d7f2cb8ad43ad768f5f92e64ea7
d447196b6934b50e626da452b9e3e064433de370f590b3a20642029829503ffd
975868018c1e0fef8bb341449621a8da5e8b4a531bfdf5213ff7ba6847f965df
52cee0deecefcd77a20ebc382f64128df05ce00b59d62d99cde70115e067f432
35d41121c3de9865ed549d1cb9b0aaa90c8ab46660e1ff2ec80043a96737db9c`);

// SFrame.exe 0x00dfb320 (128 o) : table de DÉCODAGE des noms (inverse de ENC)
const NAME_DEC = hex(`
0000000000000000000000000000000000000000000000000000000000000000
21640033372d23625a47005f25362c0043355770587e4b2b304c007900520000
44486863614d4e456e66654071592729346f53467d693850283b743900323d31
6a5e517b672e6c205675425b265d72736d6b767755785424494a7a4f00416000`);

// SFrame.exe 0x00dfb3a0 (128 o) : table d'ENCODAGE des noms (utilisée par FUN_005cf720)
const NAME_ENC = hex(`
0000000000000000000000000000000000000000000000000000000000000000
67200026772c6c4e584f00372e256500385f5d2350312d24565b0059005e0000
4b7d6a3040475329417879363945467b57623d5276746832344d286b006d612b
7e442743214a49644255607166704851334c6e6f5a697273753b7a6300543500`);

// SFrame.exe 0x00eaa4c0 : alphabet des caractères préfixe/suffixe (84 car., chaîne C)
const CHARSET = Buffer.from(
  "^&T_Nsd{xo5v`rOYV+,iIU#kCJq8$'~L0P]FeBn-Au(pXHZhwDy2}agWG7K=bQ;SRt)46l@jE%9!c1[3fmMz",
  'latin1');

// SFrame.exe 0x00dfccb4.. : extensions NON chiffrées (comparaison des 4 derniers caractères)
const PLAIN_EXT = ['.dds', '.tga', '.naf', '.nx3', '.cob', '.itc', '.itm', '.itb', '.nfm'];

const DATA_FILE_COUNT = 8;

function xorInPlace(buf, start = 0) {
  for (let i = 0; i < buf.length; i++) buf[i] ^= XOR_KEY[(start + i) & 0xff];
  return buf;
}

// Indices des deux échanges : (int)((float)len * 0.66) et (int)((float)len * 0.33)
const swapIdx = (len) => [Math.trunc(Math.fround(len) * 0.66), Math.trunc(Math.fround(len) * 0.33)];
const nextSeed = (origChar, seed) => { const n = (origChar * 17 + 1 + seed) % 32; return n === 0 ? 32 : n; };

// FUN_005cf720 : nom clair -> nom encodé (tel que stocké dans data.000)
function encodeName(plain) {
  const s = Buffer.from(plain.toLowerCase(), 'latin1');
  const L = s.length;
  if (L === 0) return Buffer.alloc(0);
  let acc = 0;
  for (const c of s) acc += c * 17 + 1;
  let seed = (acc + L) % 32; if (seed === 0) seed = 32;
  const seed0 = seed;
  for (let i = 0; i < L; i++) {
    const o = s[i]; let c = o;
    for (let j = 0; j < seed; j++) c = NAME_ENC[c & 0x7f];
    s[i] = c;
    seed = nextSeed(o, seed);
  }
  let sum = 0; for (const c of s) sum += c;
  const prefix = CHARSET[sum % CHARSET.length];
  const [i0, i1] = swapIdx(L);
  let t = s[0]; s[0] = s[i0]; s[i0] = t;
  t = s[1]; s[1] = s[i1]; s[i1] = t;
  return Buffer.concat([Buffer.from([prefix]), s, Buffer.from([CHARSET[seed0]])]);
}

// Inverse : nom encodé -> nom clair
function decodeName(enc) {
  const L = enc.length - 2;
  if (L <= 0) return null;
  const s = Buffer.from(enc.subarray(1, 1 + L));
  let seed = CHARSET.indexOf(enc[enc.length - 1]);
  if (seed < 1 || seed > 32) return null;
  const [i0, i1] = swapIdx(L);
  let t = s[1]; s[1] = s[i1]; s[i1] = t; // ordre inverse des échanges
  t = s[0]; s[0] = s[i0]; s[i0] = t;
  for (let i = 0; i < L; i++) {
    let c = s[i];
    for (let j = 0; j < seed; j++) c = NAME_DEC[c & 0x7f];
    s[i] = c;
    seed = nextSeed(c, seed);
  }
  return s.toString('latin1');
}

// FUN_005e3620 : numéro du fichier data.00N à partir du nom ENCODÉ
function dataFileIndex(enc, n = DATA_FILE_COUNT) {
  let h = 0;
  for (const b of enc) {
    let c = (b << 24) >> 24;              // char signé
    if (c >= 0x41 && c <= 0x5a) c += 0x20; // A-Z -> a-z
    h = (Math.imul(h, 31) + c) | 0;
  }
  return (Math.abs(h) % n) + 1;            // 1..8 -> data.001..data.008
}

function readIndex(dir) {
  const d = xorInPlace(fs.readFileSync(path.join(dir, 'data.000')), 0); // clé roulante sur tout le fichier
  const entries = [];
  let p = 0;
  while (p < d.length) {
    const len = d[p];
    if (p + 1 + len + 8 > d.length) throw new Error(`entrée tronquée à 0x${p.toString(16)}`);
    const enc = Buffer.from(d.subarray(p + 1, p + 1 + len));
    const offset = d.readUInt32LE(p + 1 + len);
    const size = d.readUInt32LE(p + 5 + len);
    entries.push({ enc, name: decodeName(enc), offset, size, file: dataFileIndex(enc), indexPos: p });
    p += 1 + len + 8;
  }
  return entries;
}

const isEncrypted = (name) => !PLAIN_EXT.includes(name.slice(-4).toLowerCase());

function readEntry(dir, e, fdCache = {}) {
  const fd = fdCache[e.file] ?? (fdCache[e.file] = fs.openSync(path.join(dir, `data.00${e.file}`), 'r'));
  const buf = Buffer.alloc(e.size);
  fs.readSync(fd, buf, 0, e.size, e.offset);
  if (isEncrypted(e.name)) xorInPlace(buf, 0); // index de clé = position dans le fichier ressource
  return buf;
}

const extOf = (n) => { const m = /\.([^.\\/]*)$/.exec(n); return m ? m[1].toLowerCase() : '(aucune)'; };

function stats(dir, entries) {
  const byExt = {}, byFile = {}, bytesByFile = {};
  for (const e of entries) {
    byExt[extOf(e.name)] = (byExt[extOf(e.name)] || 0) + 1;
    byFile[e.file] = (byFile[e.file] || 0) + 1;
    bytesByFile[e.file] = (bytesByFile[e.file] || 0) + e.size;
  }
  const out = { entries: entries.length, byFile: {}, byExt };
  for (let i = 1; i <= DATA_FILE_COUNT; i++) {
    const real = fs.statSync(path.join(dir, `data.00${i}`)).size;
    const l = entries.filter((e) => e.file === i).sort((a, b) => a.offset - b.offset);
    let overlaps = 0, gaps = 0, end = 0;
    for (const e of l) { if (e.offset < end) overlaps++; else gaps += e.offset - end; end = Math.max(end, e.offset + e.size); }
    out.byFile[`data.00${i}`] = { entries: byFile[i] || 0, bytes: bytesByFile[i] || 0, fileSize: real, end, overlaps, gapBytes: gaps };
  }
  return out;
}

const MAGICS = {
  dds: [Buffer.from('DDS ')], jpg: [hex('ffd8ff')], png: [hex('89504e470d0a1a0a')], bmp: [Buffer.from('BM')],
  wav: [Buffer.from('RIFF')], lua: null, cfg: null, tga: null,
};

function selftest(dir, outDir) {
  const entries = readIndex(dir);
  let ok = true;
  const fail = (m) => { ok = false; console.log('ECHEC :', m); };
  console.log(`entrées : ${entries.length}`);
  const undecodable = entries.filter((e) => e.name === null).length;
  const roundtrip = entries.filter((e) => e.name !== null && encodeName(e.name).equals(e.enc)).length;
  console.log(`noms décodés : ${entries.length - undecodable}, aller-retour encode(decode(x))==x : ${roundtrip}`);
  if (roundtrip !== entries.length) fail('aller-retour des noms incomplet');
  const st = stats(dir, entries);
  for (const [f, s] of Object.entries(st.byFile)) {
    console.log(`${f}: ${s.entries} entrées, fin=${s.end}, taille=${s.fileSize}, chevauchements=${s.overlaps}, trous=${s.gapBytes}`);
    if (s.end > s.fileSize || s.overlaps) fail(`${f} incohérent`);
  }
  console.log('30 premiers noms :');
  for (const e of entries.slice(0, 30)) console.log(`  data.00${e.file} off=${e.offset} taille=${e.size} ${e.name}`);
  const fds = {};
  const samples = ['dds', 'jpg', 'png', 'bmp', 'wav', 'tga', 'lua', 'cfg'];
  if (outDir) fs.mkdirSync(outDir, { recursive: true });
  for (const x of samples) {
    const e = entries.find((y) => extOf(y.name) === x && y.size > 0);
    if (!e) continue;
    const buf = readEntry(dir, e, fds);
    const magics = MAGICS[x];
    let verdict = 'n/a';
    if (magics) verdict = magics.some((m) => buf.subarray(0, m.length).equals(m)) ? 'OK' : 'MAUVAIS';
    else if (x === 'tga') verdict = (buf[2] === 2 || buf[2] === 10 || buf[2] === 1 || buf[2] === 3) ? 'OK (type TGA plausible)' : 'MAUVAIS';
    else verdict = /^[\x09\x0a\x0d\x20-\x7e\x80-\xff]{16}/.test(buf.subarray(0, 16).toString('latin1')) ? 'OK (texte)' : 'MAUVAIS';
    if (verdict.startsWith('MAUVAIS')) fail(`magique ${e.name}`);
    console.log(`  ${e.name} (${isEncrypted(e.name) ? 'XOR' : 'clair'}) : ${buf.subarray(0, 12).toString('hex')} -> ${verdict}`);
    if (outDir) fs.writeFileSync(path.join(outDir, path.basename(e.name)), buf);
  }
  console.log(ok ? 'SELFTEST OK' : 'SELFTEST EN ECHEC');
  process.exitCode = ok ? 0 : 1;
}

function main() {
  const [dir, cmd = 'selftest', a1, a2] = process.argv.slice(2);
  if (!dir) { console.log('usage : node rappelz_data.js <dossier_client> selftest|stats|list|extract ...'); process.exit(2); }
  if (cmd === 'selftest') return selftest(dir, a1);
  const entries = readIndex(dir);
  if (cmd === 'stats') return console.log(JSON.stringify(stats(dir, entries), null, 1));
  if (cmd === 'list') {
    const re = a1 ? new RegExp(a1, 'i') : null;
    for (const e of entries) if (!re || re.test(e.name)) console.log(`data.00${e.file}\t${e.offset}\t${e.size}\t${e.name}`);
    return;
  }
  if (cmd === 'extract') {
    if (!a1 || !a2) throw new Error('extract <regex> <dossier_sortie>');
    const re = new RegExp(a1, 'i'); const fds = {}; let n = 0;
    fs.mkdirSync(a2, { recursive: true });
    for (const e of entries) if (re.test(e.name)) {
      const safe = path.basename(e.name.replace(/[\\/]/g, '_'));
      fs.writeFileSync(path.join(a2, safe), readEntry(dir, e, fds)); n++;
    }
    return console.log(`${n} fichier(s) extrait(s) dans ${a2}`);
  }
  throw new Error(`commande inconnue : ${cmd}`);
}

module.exports = { XOR_KEY, NAME_DEC, NAME_ENC, CHARSET, PLAIN_EXT, encodeName, decodeName, dataFileIndex, readIndex, readEntry, isEncrypted };
if (require.main === module) main();
