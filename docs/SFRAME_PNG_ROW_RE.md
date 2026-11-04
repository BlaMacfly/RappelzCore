# SFrame PNG row compaction: retained-evidence review

Reviewed on **November 4, 2026**. This is a static interpretation of an already captured function, not a new Native acquisition or an additional checkpoint admission.

## Identity and observations

The retained `analyze_function` evidence for `0x4e6e3c` is
`ev_436867f89bbeca227bf7b503071108a93a431c408d9ae75789a050a3993f4f0f`.
It belongs to the original SFrame SHA-256
`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`
and Ghidra profile
`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`.

The reported inclusive body is `0x4e6e3c..0x4e7075`: **570 bytes and 194 instructions**. Independent decoding covers those reported bytes. The retained raw result and its original false qualification have not been changed.

The routine reads a row width at argument 1 offset 0 and a pixel-depth byte at offset `0xb`. Argument 2 points to an in-place row buffer. Argument 3 selects a pass. It compacts selected pixels to the beginning of that buffer, then updates width and byte count (offset 4).

Original-file reads resolve the following little-endian u32 tables in `.rdata`:

| Virtual address | Values | Meaning inferred from use |
|---|---|---|
| `0xdb1af8` | `0, 4, 0, 2, 0, 1, 0` | First pixel of each horizontal pass |
| `0xdb1b14` | `8, 8, 4, 4, 2, 2, 1` | Horizontal stride |

Depths 1, 2 and 4 extract packed samples most-significant bits first and repack them into output bytes. Other supported byte-aligned depths copy whole pixel groups. The count is `(width + stride - start - 1) / stride`, with output bytes `(new_width * pixel_depth + 7) >> 3`. For pass indices at least 6 the original routine returns without changing the row; the seventh Adam7 horizontal pass has start 0 and stride 1. A negative signed index is not rejected by that upper-bound branch; it is outside the reviewed valid input domain.

There are no direct callees in the retained dossier. It records one caller, `0x49d1ce`; this review does not establish every incoming or indirect path.

The caller was also authenticated against its retained evidence, `ev_069d14341313eb1f797041a74919215211abf455a76398061557404c6647a387`. Its 486 bytes / 146 instructions pass the original independent assembly verifier without notation issues. It contains `PNG_WRITE_*_SUPPORTED` diagnostics, initializes row info at owner offset `0xf0`, and calls the compactor with that row info, `[owner+0xdc]+1` as the row buffer, and the pass byte at owner offset `0x114`. This supports the PNG **write** interpretation independently of the table pattern.

When interlacing and transform bit 2 are enabled, the caller also selects vertical rows by pass: rows divisible by 8 for passes 0/1, rows congruent to 4 modulo 8 for pass 2, divisible by 4 for pass 3, congruent to 2 modulo 4 for pass 4, even rows for pass 5, and odd rows for pass 6. Passes 1/3/5 additionally require widths of at least 5/3/2. These branch observations describe the retained caller, not a runtime observation or the whole writer's behavior.

## Interpretation and limits

The tables and sample packing match horizontal **Adam7 PNG pass extraction**, described by the [PNG specification, section 8](https://www.w3.org/TR/png/#8Interlace). `png_do_write_interlace` is a plausible descriptive identification, not a recovered original symbol or a proven library version. This routine alone does not choose the vertical pass rows or implement a whole PNG encoder/decoder.

An independently written sample-selection model was compared with the reconstructed packing logic for widths 0 through 65, seven passes, and depths 1/2/4/8/16/24/32/48/64: **4,158 comparisons pass**. The unchanged seventh pass compares only meaningful pixel bits; unused trailing source bits are not assumed zero. These are model comparisons, not execution of SFrame or proof of all invalid-input behavior.

## Exact instruction-dialect differences

The frozen verifier reports two notation disagreements:

| Address | Original bytes | Ghidra | Independent decoder |
|---|---|---|---|
| `0x4e6ea6` | `f3 a5` | `MOVSD.REP` | `rep movsd` |
| `0x4e6ead` | `f3 a4` | `MOVSB.REP` | `rep movsb` |

Both are exact two-byte repeat-prefixed string-move instructions. This explains those particular disagreements; it does not relax the frozen verifier or prove arbitrary operand equivalence elsewhere.

## Checkpoint boundary

Checkpoint 18 remains **3,659 qualified dossiers / 1,193,745 deduplicated body bytes**. The old source-only obligations for this acquisition remain open: no fresh REA `read_bytes` or `address_to_file_offset` evidence was acquired in this review. The byte checks here are explicitly independent original-file observations. The report must not be counted as a newly admitted function, a new Evidence ID, or completed decompilation of SFrame.
