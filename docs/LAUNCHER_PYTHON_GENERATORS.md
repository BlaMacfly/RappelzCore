# Launcher Python generators: seven bounded partial results

Verified **2026-11-02**. A reviewed generator model now produces syntax-valid **partial text for seven previously failing application code objects**. These are seven new analysis operations, with 83 other results retained from pass 02. No original launcher, extracted module or reconstructed source was executed, and no target/reconstructed source was compiled.

This report extends [the earlier Python reconstruction checkpoint](LAUNCHER_PYTHON_RECONSTRUCTION.md), which remains byte-identical at SHA-256 `969adb1f736cc0220ac6c4384616e2e85a37801504d51855a740433e3dc391a8`. Its pass-01/pass-02 counts describe that historical checkpoint; this separate document records the later seven-selector pilot without rewriting its evidence.

## Scope and origins

The subject is still `NoLimits_Launcher_FR`, the **90-code-object** application payload inside `NoLimits_Launcher_3.exe`. Its 94,047-byte marshaled payload has SHA-256 `b591a4762b5d6d2516dcb70bf2437131e5d0ca68fa85b48827519d44b00ab79d`. The original launcher has SHA-256 `8b39b900d9ccdb687192e5a249d58984858991dd3cf9fd211147161223cd4fa1`.

A code object can represent a module/class body, ordinary callable, lambda or generator; it is not a source-function count. This application trial is a subset of the **383 Python payloads / 11,102 code objects** previously disassembled. The **68 embedded x64 PE files** are another scope entirely. None of these denominators measures reconstructed gameplay.

| Origin | Objects | Classification | AST-valid fragments |
|---|---:|---|---:|
| Newly produced in pass 03 | **7** | 7 partial | **7** |
| Inherited from pass 02, not rerun | **83** | 22 syntax-only labels, 61 partial | **82** |
| Reference overlay of those two origins | **90** | 22 syntax-only labels, 68 partial | **89** |

All seven pass-03 operations ran once, in order **3, 25, 28, 2, 27, 29, 30**. There were no timeouts or failed child operations. The reference overlay has 90 retained text fragments, including one inherited fragment that still fails syntax parsing. It is not a claim that 90 producers ran again, that 89 functions are complete, or that the application has been rebuilt. The 22 syntax-only labels are inherited; pass 03 adds **zero completed-success labels**.

## What changed

The isolated tool remains [depyf 0.20.0](https://pypi.org/project/depyf/0.20.0/) with an authored, restricted adapter. The analysis used local **CPython 3.12.10**; matching bytecode magic does not establish the packaged runtime's exact patch release. Version labels for the game and reconstruction remain separate in [VERSIONS.md](VERSIONS.md).

The new model accepts only the normal **`YIELD_VALUE → RESUME 1 → POP_TOP`** pattern, where the resumed value is discarded. It separates generator creation from the external initial-resume entry. At suspension it removes the yielded value and preserves the lower stack; at resumption it introduces a distinct incoming-value token, which the following `POP_TOP` consumes. These are symbolic analysis events, not creation or resumption of actual target generators.

The model follows the pinned [CPython 3.12.10 bytecode implementation](https://github.com/python/cpython/blob/v3.12.10/Python/bytecodes.c). In this version, the `YIELD_VALUE` argument describes exception-block depth rather than stack depth. The protocol also distinguishes the creation return from an ordinary function return.

Used resume values, `yield from`, async generators and unproven user handlers remain rejected. The existing core's skipped StopIteration-error intrinsic is replaced with an explicit unresolved exceptional-path diagnostic if traversed. Recognizing the standard tail does not establish full [PEP479 conversion behavior](https://github.com/python/cpython/blob/v3.12.10/Python/intrinsics.c), including exception cause/context preservation.

## Fixtures, guards and identities

Before the target pilot, **14 checks** passed on authored fixtures: six positive syntax experiments, four unsupported-feature cases, protocol transitions and execution protections. They covered plain and repeated yields, nested generators, closures and missing-default qualification. Fixture source was compiled into test bytecode as data; neither fixture functions nor reconstructed outputs were executed.

The seven target operations retained the pass-02 guards and bounds: **10,000 dispatches, 16 entries per range, 256 retained trace events and five seconds per object**. Each child ran with Python `-I -S -B`. General execution/evaluation, protected function construction, executable compilation, network/process creation and writes outside the new audit were blocked.

Independent verification checked **42 passed results of six distinct guard self-tests**, with unchanged counters after analysis and no unexpected guard event. This is not 42 different protections or an extra set of recovered functions. The seven dispatch counts were **17, 19, 19, 47, 50, 26 and 31**, totaling 209.

| Selected producer | Role | Generator contexts observed | Result |
|---|---|---|---|
| 3 | Nested generator used by the elevation helper | 3 | Partial, AST-valid |
| 25 | Generator in the data-file check | 25 | Partial, AST-valid |
| 28 | Generator nested under object 27 | 28 | Partial, AST-valid |
| 2 | Ordinary caller constructing object 3 | 3 | Partial, AST-valid |
| 27 | Outer generator constructing object 28 | 27 and 28 | Partial, AST-valid |
| 29 | Generator with free variable `name` | 29 | Partial, AST-valid |
| 30 | Generator with free variable `state` | 30 | Partial, AST-valid |

The **eight context occurrences represent six canonical generator identities**, alongside one ordinary caller. Contexts 3 and 28 appear both independently and under their callers; they are not additional unique successes. Objects 25 and 28 share bytecode bytes but retain distinct code paths, line tables and parent scopes. Their output and operation identities remain separate.

## Remaining obligations and native limits

Every new result retains partial qualifications. All eight contexts preserve their exception-table records and explicitly report incomplete exception coverage. `throw`, `close`, finalization and PEP479 remain unresolved. The `name` and `state` cells are named but their live bindings are not recovered.

The **14 parent-supplied default-definition obligations** remain in the inherited pass-02 records; nothing in this generator repair supplies their missing values. Parent iterator timing also remains open: the outer iterable is obtained at generator creation while the body stays lazy, and helper-function reconstruction can introduce an additional implicit `iter()` edge. Parent consumer order and short-circuit behavior must be preserved rather than replaced with eager materialization.

This Python result adds no Native dossier, verified native-body byte or native census row. The [frozen native module checkpoint](GLOBAL_DECOMPILATION_MODULES.md) keeps separate provider-defined boundaries, explicit Audiere/MSS failures and retained-text/graph limitations. The 68 embedded PE targets remain outside that checkpoint's native table. Python syntax results cannot close those obligations or provide a global game-completion percentage.

The practical improvement is inspectable normal-path text where the seven selectors previously stopped on unsupported generator handling. The next work is to qualify exceptional edges, parent iterator/cell bindings and definition context against the retained bytes and new authored fixtures. Original-source recovery, semantic equivalence and runtime compatibility remain unverified.

## Preservation and local proof pins

The independent verifier rehashed **31 source pins plus the interpreter**, all **846 pass-02 files**, the 21 copied upstream tool files and all 83 inherited result records. Original launcher bytes, settings, databases and shared tools were unchanged. No native engine was needed. Raw payloads, generated code and detailed proofs remain local.

The ignored audit directory is `client-data/audit/2026-11-02-global-decompilation-ec98a7d4/launcher-decompile-generator-03/`. Its report distinguishes newly produced records from inherited references. These authored local receipts are not REA Evidence or proof of semantic correctness.

| Pin | SHA-256 / digest |
|---|---|
| Reviewed generator-model source | `0ca6b964e3bc38253faf54a6232a1093e73cd843713a53cb7bde6bf0cc6a132a` |
| Pass-03 profile digest | `4e42513e76f74c8394b16c5e282c75404c4b9dfe70569ceb51e508c8b186d210` |
| Producer `report.local.json` | `4cf3d6dc890d915ee54790466a0edae7704a0cc079d2bb354eee98827df48418` |
| `independent-report.local.json` | `914ea5b1d5c254cd80e062dc5aaa80318e7f312d4773efebcfcee3ba31293a1a` |
| `verification.local.json` | `dadf011df362b9057244b9f72d4195e83cef837d5c6cbbfa85b27637b80b2738` |
| `postrun-manifest.local.json` | `2b9e64fa911ebbafd7aeb27130560e602a74ce5294148a10080becb72cbdfa25` |
