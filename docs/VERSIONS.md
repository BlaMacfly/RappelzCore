# Versions and original game target

Checked **2026-10-31** against the local manifests and supplied original client. RappelzCore/RappelzRust are a reconstruction in development; the version of our software and the original game's labels are tracked separately.

| Component / evidence | Version or identifier | Meaning |
|---|---|---|
| RappelzRust | **0.1.0**, Windows x64, development | Current `[workspace.package].version`; `RappelzRust/client/Cargo.toml` inherits it. This is our client version. |
| RappelzCore authserver/worldserver | **0.1.0**, development | The two server packages inherit the same workspace version. |
| Supplied original game | **Rappelz NoLimits**, customized x86 client | The original game used as the reconstruction target. |
| Original launcher/package | **1.3.60** | Read from the supplied `Rappelz NoLimits/version.txt`; it is a launcher/package label. |
| Original SFrame file/product metadata | **9.1.0.0 / 9.1.0.0** | PE version-resource declarations, checked statically; these may be inherited or stale. |
| Embedded original build/debug directory | **Source9.5.2** | Build lineage indicator. It does not establish an official Epic 9.5.2 release or complete compatibility. |
| Original network-version literal / current implemented profile | **201507080** | Present in the supplied SFrame and used by the reconstructed protocol. Full original-client interoperability remains unverified. |
| Exact official game Epic / patch | **Not established** | The supplied labels conflict; an exact official release must not be guessed. |

The current original artifact is `Rappelz NoLimits/SFrame.exe`, PE32 x86, **12,481,536 bytes**, SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**. This identity and the observed packet layouts define the compatibility target. Read the [binary identity investigation](CLIENT_REVERSE_ENGINEERING.md), [protocol scope](NETWORK_PROTOCOL.md) and [fresh progression decompilation](SFRAME_PROGRESSION_RE.md) for the supporting evidence and limits.

Both `RappelzCore` and `RappelzRust` Git branches carry this table and the English/French README summary. The branch commit identifies the precise source revision; `0.1.0` alone does not identify a particular development build or imply a published release tag. Updating this documentation does not bump the package version, reinterpret character data or establish an official original-game patch.
