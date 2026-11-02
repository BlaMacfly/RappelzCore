# Original uninstall UPX stub: bounded static findings

This report concerns the **original packed** `uninstall.exe` supplied in the NoLimits **1.3.60** package. It is separate from the unpacked PE derivative and its 20,510-entry native census. The original stub has **not** received a complete Native/function census. No installer, game or recovered implementation was executed for this static study.

| Artifact | Size | SHA-256 |
|---|---:|---|
| Original packed `uninstall.exe` | 1,344,512 bytes | `5ffd4b20dccfb84c8890abdb780184a7651e760aefba4ab0c6fba5b2a81f97d9` |
| Separately produced UPX PE13 derivative | 3,805,696 bytes | `5ef772bfd3c163011967a6c5f8eacb53df2f653337594b0529e548f2296e76d1` |

The derivative was produced from a copy with a pinned official UPX tool. It is a **nonexact legacy PE13 reconstruction**, not an exact restoration of the historical original file. Its census cannot satisfy the packed original's remaining scope. See the [module checkpoint](GLOBAL_DECOMPILATION_MODULES.md) and [version labels](VERSIONS.md); SFrame's 9.1.0.0 / Source9.5.2 labels do not identify this installer's exact original build.

## Entry control flow and physical bytes

The original PE declares preferred image base `0x400000` and entrypoint **`0x7c2d10`**, at file offset **1,310,992**. A retained 758-byte read spans file offsets `[1310986, 1311744)` and preferred VAs `0x7c2d0a`–`0x7c2fff`. This is a bounded read, not a whole function boundary.

The entry-directed static graph has **184 instruction decodings**, without decoder failure. Their sizes sum to 436 bytes; the physical union is **432 bytes**, because some decodings overlap. The qualified inclusive intervals are:

| Preferred-address interval | Meaning |
|---|---|
| `0x7c2d10`–`0x7c2d1e` | Entry-directed instruction bytes |
| `0x7c2d20`–`0x7c2ec0` | Remaining entry-directed byte union |

Three overlapping decode pairs involve the instruction starting at `0x7c2e62` and starts at `0x7c2e63`, `0x7c2e64` and `0x7c2e65`. Their unique overlap occupies four bytes. A future disassembler import must preserve those conflicts explicitly; 184 decoding starts are not 184 disjoint instructions in one conventional listing.

Both conditional successors and normal post-call fallthroughs are considered. They are possible static paths conditional on called routines returning. Runtime feasibility and termination are unobserved. The unused window portions, including `0x7c2d0a`–`0x7c2d0f`, `0x7c2d1f` and `0x7c2ec1`–`0x7c2fff`, remain outside this qualification. They are not proved padding or proof that other entrypoints do not exist.

The compressed stream occupies file offsets **`[1024, 1310986)`**, or 1,309,962 bytes. It was not disassembled as code. An executable-section flag does not turn compressed bytes into a code census.

## Declared APIs and bounded dataflow

Original import declarations and bounded register dataflow identify five call sites through four declared KERNEL32 symbols:

| Call site | Slot or register provenance | Declared API |
|---|---|---|
| `0x7c2e4a` | IAT `0x7cac10` | `LoadLibraryA` |
| `0x7c2e68` | IAT `0x7cac14` | `GetProcAddress` |
| `0x7c2e79` | IAT `0x7cac24` | `ExitProcess` |
| `0x7c2e96` | EBP loaded at `0x7c2e7f` from IAT `0x7cac18` | `VirtualProtect` |
| `0x7c2eab` | Same bounded EBP provenance | `VirtualProtect` |

Declarations do not identify runtime API addresses, hooks, loaded module contents, load success or actual side effects. Continuation uses normal-return and Win32 preserved-register assumptions. The explicit stack save/restore recovers ESI `0x401000` on the bounded normal tail; no live register trace was taken.

Static initialization selects compressed input ESI **`0x683000`** and output EDI **`0x401000`**. The packed header declares an uncompressed extent of **3,932,862 bytes**, ending at `0x7c12be`. This is a declared extent, not an observed write range or proof of decompression correctness. A runtime import-descriptor candidate at `0x7be000` is inferred from dataflow; compressed file bytes are not interpreted as its reconstructed runtime contents.

The filter-related counter 94,025, header filter ID `0x26` and header CTO 100 are recorded. Their relationship to complete filter semantics is an inference. The memory-write list covers explicit Capstone WRITE operands; implicit PUSH/PUSHAD stack writes are not an exhaustive write census.

## Header changes and final handoff

Retained instructions address header bytes **`0x400227`** and **`0x40024f`**, matching original file offsets `0x227` and `0x24f`. Clearing bit `0x80` in these most-significant characteristic bytes corresponds to clearing **`IMAGE_SCN_MEM_WRITE` (`0x80000000`)**. A VirtualProtect-style sequence uses page base `0x400000`, size 4,096 and new-protection argument 4, followed by an old-protection restoration sequence. Successful API execution and effective page protection are unobserved.

The final direct transfer is **`0x7c2ebc` → `0x5bba47`**. Its destination equals the derivative's declared entrypoint. That original preferred address is in UPX0, which has **zero raw file bytes**. It has no original-file instruction bytes to validate as an ordinary body here. The matching address does not prove that runtime reached it or permit merging original and derivative censuses.

COFF flags `0x103` include RELOCS_STRIPPED and the relocation directory is empty, while DLL characteristics `0x8140` include DYNAMIC_BASE. This study uses preferred-address mappings only. Actual load base and relocation handling remain unknown. Delay-import directory 13 points into the compressed stream, so it was not decoded as reconstructed runtime descriptors.

## Evidence and remaining scope

Independent retained-byte verification passed **20 tests**, checked preserved source hashes and revalidated the report's 72 source pins, including nested primary-source lineage. Verification SHA-256: **`255283e5a40c11c1a92a72d36bbe2524f78f89c2dbd7642d430afbc3c5076af0`**. Detailed CFG report SHA-256: **`3bbdd5cf6edc3912a41a5222a8e3c248dbb74f1d1901d1464acee8766e4a4e1c`**. Private evidence remains under `client-data/audit/2026-11-02-global-decompilation-ec98a7d4/original-UPX-stub-static-01`; no raw reconstructed source is published here.

The pinned [UPX PE32 implementation](https://github.com/upx/upx/blob/7c7bea44146c7ad6e17cb2d0bb9566805b44a91a/src/p_w32pe_i386.cpp) supports the current header/import/entrypoint interpretation. It does not establish exact source identity for the binary's `3.05` literal or identify its exact historical packer build.

This audit adds **zero Native dossiers**, **zero original native censuses** and **zero formal SFrame body bytes**. A future original-stub route needs a separately reviewed read-only import with automatic analysis disabled, restricted to the qualified entry graph with overlapping decodings handled explicitly. The public default REA import is not claimed to support that restriction already. Complete original-stub function discovery, runtime decompression behavior and whole-program semantics remain open.
