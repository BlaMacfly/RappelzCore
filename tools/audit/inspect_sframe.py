#!/usr/bin/env python3
"""Read-only PE identity/import/string audit, using only Python's standard library.

This does not load the PE or execute its code. Embedded versions and timestamps
are declarations, not a guarantee of an official retail release.
"""

import argparse
import datetime
import hashlib
import json
import re
import struct
from pathlib import Path


def inspect(path):
    path = path.resolve(strict=True)
    data = path.read_bytes()

    def span(offset, length):
        if offset < 0 or length < 0 or offset + length > len(data):
            raise ValueError(f"Out-of-bounds PE data at {offset:#x}, length {length}")
        return data[offset:offset + length]

    def u16(offset):
        return struct.unpack("<H", span(offset, 2))[0]

    def u32(offset):
        return struct.unpack("<I", span(offset, 4))[0]

    def cstring(offset):
        end = data.find(b"\0", offset, min(len(data), offset + 16384))
        if offset < 0 or end < offset:
            raise ValueError(f"Invalid PE string at {offset:#x}")
        return data[offset:end].decode("ascii", errors="backslashreplace")

    if span(0, 2) != b"MZ":
        raise ValueError("Not an MZ image")
    pe = u32(0x3C)
    if span(pe, 4) != b"PE\0\0":
        raise ValueError("Not a PE image")
    coff = pe + 4
    machine, count = u16(coff), u16(coff + 2)
    timestamp = u32(coff + 4)
    optional = coff + 20
    magic = u16(optional)
    if magic not in (0x10B, 0x20B):
        raise ValueError("Unsupported PE optional header")
    is_64 = magic == 0x20B
    image_base = (struct.unpack("<Q", span(optional + 24, 8))[0]
                  if is_64 else u32(optional + 28))
    section_table = optional + u16(coff + 16)
    sections = []
    for i in range(count):
        offset = section_table + 40 * i
        name = span(offset, 8).split(b"\0")[0].decode("ascii", errors="replace")
        sections.append({"name": name, "virtual_size": u32(offset + 8),
                         "rva": u32(offset + 12), "raw_size": u32(offset + 16),
                         "raw_offset": u32(offset + 20)})

    def file_offset(rva):
        candidates = [s["raw_offset"] + rva - s["rva"] for s in sections
                      if s["rva"] <= rva < s["rva"] + s["raw_size"]]
        if not candidates and rva < u32(optional + 60):
            return rva
        if len(candidates) != 1:
            raise ValueError(f"Unmapped or ambiguous RVA {rva:#x}")
        span(candidates[0], 1)
        return candidates[0]

    def location(offset):
        mappings = [s for s in sections
                    if s["raw_offset"] <= offset < s["raw_offset"] + s["raw_size"]]
        result = {"file_offset": hex(offset)}
        if len(mappings) == 1:
            result["va"] = hex(image_base + mappings[0]["rva"]
                               + offset - mappings[0]["raw_offset"])
        return result

    directories = optional + (112 if is_64 else 96)
    import_rva, import_size = u32(directories + 8), u32(directories + 12)
    imports = []
    if import_rva:
        descriptor = file_offset(import_rva)
        for _ in range(import_size // 20):
            fields = struct.unpack("<IIIII", span(descriptor, 20))
            if not any(fields):
                break
            thunk = file_offset(fields[0] or fields[4])
            width, high_bit = (8, 1 << 63) if is_64 else (4, 1 << 31)
            symbols = []
            for _ in range(65536):
                value = int.from_bytes(span(thunk, width), "little")
                if value == 0:
                    break
                symbols.append({"ordinal": value & 0xFFFF} if value & high_bit
                               else {"name": cstring(file_offset(value) + 2)})
                thunk += width
            else:
                raise ValueError("Import thunk table exceeds audit budget")
            imports.append({"dll": cstring(file_offset(fields[3])), "symbols": symbols})
            descriptor += 20

    # VERSIONINFO is located through its resource tree, not by scanning for a
    # signature that could also occur in unrelated bytes.
    resource_rva = u32(directories + 16)
    versions = []
    if resource_rva:
        root = file_offset(resource_rva)

        def entries(relative):
            base = root + relative
            total = u16(base + 12) + u16(base + 14)
            if total > 4096:
                raise ValueError("Resource directory exceeds audit budget")
            return [(u32(base + 16 + i * 8), u32(base + 20 + i * 8))
                    for i in range(total)]

        for kind, type_target in entries(0):
            if kind != 16 or not type_target & 0x80000000:
                continue
            for name, name_target in entries(type_target & 0x7FFFFFFF):
                if not name_target & 0x80000000:
                    raise ValueError("Unexpected VERSIONINFO resource shape")
                for language, leaf in entries(name_target & 0x7FFFFFFF):
                    if leaf & 0x80000000:
                        raise ValueError("Unexpected nested VERSIONINFO resource")
                    entry = root + leaf
                    payload = file_offset(u32(entry))
                    blob = span(payload, u32(entry + 4))
                    signature = blob.find(b"\xbd\x04\xef\xfe")
                    if signature < 0 or signature + 24 > len(blob):
                        raise ValueError("Missing VS_FIXEDFILEINFO")
                    fixed = struct.unpack_from("<6I", blob, signature)

                    def version(ms, ls):
                        return ".".join(map(str, [ms >> 16, ms & 65535, ls >> 16, ls & 65535]))

                    versions.append({"name_id": name, "language_id": language,
                                     "file_version": version(fixed[2], fixed[3]),
                                     "product_version": version(fixed[4], fixed[5]),
                                     "payload": location(payload),
                                     "payload_bytes": len(blob),
                                     "payload_sha256": hashlib.sha256(blob).hexdigest()})

    anchors = []
    pattern = re.compile(rb"[\x20-\x7e]{5,}")
    selectors = (b"source9.5.2", b"201507080", b"Jul 31 2026", b"17:53:58",
                 b"net_debug.txt", b".pdb", b"lua5.1.dll")
    for match in pattern.finditer(data):
        if any(selector.lower() in match.group().lower() for selector in selectors):
            anchors.append({**location(match.start()), "text": match.group().decode("ascii")})

    return {"path": str(path), "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
            "machine": hex(machine), "format": "PE32+" if is_64 else "PE32",
            "architecture": {0x14C: "x86", 0x8664: "x86_64"}.get(machine, "unknown"),
            "coff_timestamp_raw": timestamp,
            "coff_timestamp_utc": datetime.datetime.fromtimestamp(
                timestamp, datetime.timezone.utc).isoformat(),
            "image_base": hex(image_base),
            "entry_va": hex(image_base + u32(optional + 16)),
            "linker_version": f"{data[optional + 2]}.{data[optional + 3]}",
            "sections": sections, "versions": versions,
            "normal_imports": imports, "string_anchors": anchors,
            "limits": ["Static bytes only; the target is never loaded or executed.",
                       "COFF timestamps, version resources and paths may be stale or modified.",
                       "Normal imports only; dynamic LoadLibrary/GetProcAddress behavior is not traced.",
                       "Embedded strings do not prove that a runtime code path uses their contents."]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect(args.path), indent=2, ensure_ascii=False))
