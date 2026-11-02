# Launcher Python reconstruction: bounded static results

Checked **2026-11-02**. This checkpoint records partial, static reconstruction of the application script packaged inside `NoLimits_Launcher_3.exe`. The launcher application has not been rebuilt, and no recovered output has been compiled or executed. The two passes below measure emitted text, diagnostics and syntax qualification; they do not establish original source, equivalent behavior or implemented game features.

## Scope and identities

The supplied launcher is **18,804,279 bytes**, SHA-256 `8b39b900d9ccdb687192e5a249d58984858991dd3cf9fd211147161223cd4fa1`. Its **18,331,703-byte** CArchive package was extracted into a separate local audit with source extents and hashes. Package labels and original-game versions remain separately documented in [VERSIONS.md](VERSIONS.md).

| Layer | Verified scope | What this establishes |
|---|---:|---|
| CArchive | 1,012 entries | Bounded extraction and retained member identities |
| Python static inventory | 383 code payloads, 11,102 unique code objects, 1,115,106 wordcode entries | Metadata and disassembly; six records retain presentation limitations |
| Embedded native inventory | 68 PE member files, all x86_64 | Header inventory; no native-function decompilation coverage follows from this count |
| Application reconstruction trial | **90 code objects / 20,397 wordcode entries** from `NoLimits_Launcher_FR` | The sole subject of both text-reconstruction passes |

A code object is not necessarily a source function. This application inventory includes module/class bodies, methods, lambdas and nested generators. Payload, code-object, source-file and native-PE counts must not be added together or used as interchangeable completion denominators. Inline `CACHE` entries also contribute to wordcode counts without being separate execution operations.

The application payload is a **94,047-byte** marshaled code graph, SHA-256 `b591a4762b5d6d2516dcb70bf2437131e5d0ca68fa85b48827519d44b00ab79d`. Its bytecode magic `cb0d0d0a` agrees with the local analysis interpreter, **CPython 3.12.10**, Windows x64. That format match does not identify the packaged runtime's exact patch release.

## Comparison of the two passes

Both passes use the same 90 original application code objects. Pass 01 used a guarded, restricted depyf core. Pass 02 changed the local adapter, execution bounds and qualification rules after static investigation and authored-fixture checks.

| Result classification | Pass 01 | Pass 02 |
|---|---:|---:|
| Syntax-only success | 42 | 22 |
| Partial | 41 | 61 |
| Failed | 6 | 7 |
| Wall-clock timeout | 1 | 0 |
| Total attempted code objects | **90** | **90** |
| Fragments accepted by `ast.parse` — overlapping the rows above | **82** | **82** |

Each pass emitted 83 fragments; one partial fragment in each pass failed syntax parsing. A partial fragment can parse successfully while omitting a handler, stopping before an unsupported instruction, or lacking definition context.

The fall from 42 to 22 success labels reflects stricter qualification, not a measured functional regression: **21 previous syntax-only successes became partial**, while one previous failure became syntax-only. Runtime behavior was not tested. In particular, nine previous success labels hid default-argument obligations that the second pass now preserves explicitly. These figures do not describe complete decompilation of the application or the wider Python inventory.

## Tool choice and actual adapter repairs

The analysis acquired the pure-Python [depyf 0.20.0 wheel](https://pypi.org/project/depyf/0.20.0/) into an isolated audit directory, without installing it. Its [upstream decompiler](https://github.com/thuml/depyf/blob/master/depyf/decompiler.py) is aimed at function reconstruction and has explicit unsupported constructs. Only reviewed core modules were loaded; package initialization, PyTorch integration and debugging hooks were bypassed. The wheel and upstream source bytes were preserved.

The local adapter now implements:

- **Bounded progress:** at most 10,000 dispatched operations, 16 entries per range and 256 retained trace events. Invalid subranges, transfers below a range start and exits without progress produce diagnostics. Trace entries precede dispatch and termination. A five-second per-object wall limit remains an independent safeguard.
- **Iterator lifetime:** a distinct iterator slot remains on the symbolic stack during a `FOR_ITER` body. Exhaustion is modeled separately, then `END_FOR` removes its two values. This agrees with local `dis.stack_effect` observations and the [CPython 3.12 instruction semantics](https://docs.python.org/3.12/library/dis.html). Early-return `SWAP 2` therefore has the required iterator/value pair.
- **Proven context boundaries:** `BEFORE_WITH` is accepted only when one exception-table entry links the body to the expected handler and the normal cleanup sequence matches. An ambiguous or unsupported pattern remains an error. The adapter does not invent general `try/except` support.
- **Conservative region state:** original non-cache instructions remain traceable; live branch stacks and terminal transfers are distinguished. Unequal live stack depths remain unresolved. `RETURN_GENERATOR` is not treated as a normal function return, and Python 3.12 generator support remains explicitly incomplete.

These are local adapter changes, not an upstream depyf release or a general-purpose Python 3.12 decompiler. Exception-table interpretation follows the [CPython 3.12.10 design notes](https://github.com/python/cpython/blob/v3.12.10/Objects/exception_handling_notes.txt); arbitrary handler and control-flow reconstruction is still outstanding.

## Validation and focused outcomes

**12 authored-fixture checks** passed before the new application attempts. They cover stack effects, early returns, loop exits, context-region selection/rejection, unsupported generators, dispatch/reentry limits and bounded traces. The fixtures were compiled into bytecode as test data; their functions and reconstructed outputs were not executed.

Objects **4, 41, 42 and 64** were attempted first. Their verified receipts were then reused while the remaining 86 objects were processed, so the four producers were not repeated. All **90 attempts have receipts for the same six execution-guard self-tests**. These are repeated checks of six protections, not 540 distinct test cases. No unexpected execution-guard event occurred. The maximum observed dispatch count was **601**, with no wall timeout.

| Object / static identity | Pass 02 result and practical limit |
|---|---|
| 4 — `_resolve_auth_ip` | `SWAP 2` proceeds with the iterator slot preserved. The output remains **partial** because exception handlers are unresolved. No name-resolution endpoint was called. |
| 41 — `_resolve_chain` | An explicit **nonprogress diagnostic** stops the backward transfer at byte offset 754 toward offset 452 after 157 dispatches. Retained partial text is not a successful decompilation. |
| 42 — `_apply_delta_chain` | Previously failed on `SWAP 2`; now emits syntax-valid text. The label remains **syntax-only**, with no behavioral-equivalence claim. |
| 64 — `_fetch_news` | Completes the bounded attempt after 106 dispatches and remains **partial**. The context body is linked to offsets 342–444 and handler 638; outer exception behavior is unresolved. No news request was made. |

The `_fetch_news` CFG contains **141 non-cache operations, 36 blocks, 58 normal/exception edges and ten exception regions**. Its original timeout had no sampled stack. The static investigation identified a credible stale-loop-bound cycle; pass 02 verifies the corrected region mapping and bounded outcome, rather than proving retrospectively where the old process stopped.

The seven remaining failures are caused by unsupported Python 3.12 generator handling, including a caller constructing a nested generator: objects **2, 3, 25, 27, 28, 29 and 30**. They must not be relabeled as recovered functions.

## Unresolved obligations and preservation

The static obligation ledger covers all 90 objects. It records **14 definitions with parent-supplied default arguments**, **19 objects needing closure context**, and **42 objects containing exception regions** requiring complete review. The nine previously unqualified success labels with missing defaults are objects **55, 66, 67, 68, 71, 72, 84, 85 and 86**; all remain partial in pass 02. A code object's argument names alone do not recover its parent's default values or closure bindings.

Execution protections disable general `exec`/`eval`, prevent protected code objects from becoming callable functions, permit compilation only for AST parsing during target analysis, deny network/process creation, and restrict writes to the new audit. The adapter uses literal parsing for constant data. No target, extracted module or reconstructed source was executed; no target/reconstructed source was compiled. Existing game files, databases and shared tools were unchanged. Hash checks preserve the original launcher, extraction evidence, all **479 pass-01 files**, and **21 copied upstream tool files**. Raw code and private audits remain local.

The next concrete obligation is a reviewed Python 3.12 generator suspension/resumption model, with authored static fixtures for nested generators and cleanup edges. It must preserve the execution guards and default/closure obligations before any further reconstruction pass. A guard replacing a timeout, or an AST parser accepting text, cannot discharge that obligation.

## Reproducible local pins

Proofs are retained under the ignored audit root `client-data/audit/2026-11-02-global-decompilation-ec98a7d4/`, in `launcher-static-extraction-01`, `launcher-decompile-depyf-01`, `launcher-reconstruction-obligations-01` and `launcher-decompile-depyf-02`. For the obligation analysis, use revision `local-02` and its `active-analysis.local.json`; the superseded first draft incorrectly included cache rows in one opcode classification.

| Pin | SHA-256 / digest |
|---|---|
| depyf 0.20.0 wheel | `d31effad4261cebecb58955d832e448ace88f432328f95f82fd99c30fd9308d4` |
| Local CPython executable | `4d6f5f81a4bca11191c4c7c6b43632694d0a4ce74e068619d8fdc161d469859a` |
| Pass-02 adapter source | `f9838e0d29c356aa698c864ee5762c25a8239dcc61d8dedd80cc61c5e9d77d71` |
| Pass-02 guarded child source | `8b38596107753f7767bc879b6a32d968bb8144e86ea5489f218a30ddc8e4f433` |
| Pass-02 profile digest — separate from its JSON-file hash | `6a593aa63d636d8a1f582551d19efc970c6344e0a4d07a680223c6ecbe191726` |
| Pass-02 `verification.local.json` | `0ff69faf57753e42a0f4899d1576e26a39b54f8da08c25d675ad05d6de5b171c` |
| Pass-02 `evidence-manifest.local.json` | `71f8137485746f0e3bb7837ea90d640098b8d19f9f63bff891cca87ceb811992` |

The verifier pins the report, comparator, source files, output bytes, traces and guard receipts. These local authored receipts are distinct from REA Evidence and do not authenticate semantic correctness. This checkpoint adds no verified source-complete function, runtime compatibility result or reconstructed game feature to the [global decompilation metrics](GLOBAL_DECOMPILATION.md).
