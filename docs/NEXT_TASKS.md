# Next tasks / restart point

Last completed cycle: 2026-10-31. Read [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md) first. Do not restart or reconstruct subsystems already present.

1. **Extend the verified two-client regression.** [COMBAT_VALIDATION.md](COMBAT_VALIDATION.md) covers kills and XP/JP level-up; [RESPAWN_VALIDATION.md](RESPAWN_VALIDATION.md) adds repeated kill/respawn. [REVIVAL_VALIDATION.md](REVIVAL_VALIDATION.md) now covers lethal player damage, automatic revival, observer position/rendering after 3.5 seconds before QUERY, Lua death/alive events and saved offline state. Server tests cover range/layer gates, once-per-kill rewards and old/new/continuing teleport viewers. Next coverage: more observers, native range/layer transitions, repeated deaths with simultaneous aggressors and group reward rules when implemented. Manual resurrection/ghost/death-penalty behavior is not implemented. Keep fixture preparation restricted to new offline records; retain earlier fixtures unchanged.
2. **Complete addon host edge cases.** Physical-keyboard verify F9 and `/reload`: computer-use mouse worked, injected keyboard events did not establish those paths. Add a client-state adapter test independent of a macroquad graphics context. Review error behavior on corrupt preferences and multi-addon/multi-file save failures; consider periodic saves and a worker/process if untrusted third-party addons become a target. Current hook quotas cannot preempt long native C calls. Preserve startup/event error isolation and path containment tests.
3. **Expand Lua UI deliberately.** Add image-backed textures through the existing Assets cache, font selection/wrapping, useful tooltip APIs, more XML schema pieces/anchor overloads and OnUpdate only with measured quotas. Implement inter-addon exports if needed; separate VMs currently do not share globals. Do not remove Lua 5.1 pcall restrictions without proving quotas cannot be swallowed. Update LUA_API_COMPATIBILITY for every new capability.
4. **Establish original-client protocol compatibility.** CLIENT_REVERSE_ENGINEERING records the SHA-bound target and conflicting identifiers. Ghidra's new overview timed out; retained snapshot has zero native query answers. Reuse `client-data/proto-re/*.c` and existing docs/re findings. Obtain a controlled authorized local capture or focused decompilation to resolve KSA and version/login packet differences versus rzu/RZEmulator. Do not infer an official Epic/patch from one string. Never connect to a third-party service to test this without user scope.
5. **Replace provisional combat/data formulas.** Decode/validate BasicStat/Exp and related RDBs, then connect true stat/experience/skill state. Existing melee/reward behavior is provisional. Work toward server skills, quests, summon/pet state, parties and corresponding real client events. Keep Rappelz-specific data in explicit tables/migrations rather than force incompatible systems into WoW tables.
6. **Strengthen archive integrity and import robustness.** Expansion and sparse patch priorities are now implemented/tested. Add bounded malformed-MPQ validation, integrity manifest/checksum coverage and error reporting for directory enumeration. Existing verify compares all 125,212 packaged original non-DB assets; generated DBC and loose Resource inputs need distinct validation. No binary-delta/tombstone patch semantics exist.
7. **Continue rendering where verified.** Water, sky/fog, effects/grass, bone attachments, lighting and fuller skill/inventory UI remain. Reuse rz-terrain, rz-model, rz-texture and rz-ui. Original converted assets remain local and excluded from git.
8. **Keep the two Git branches consistent.** User-authorized publication uses isolated worktrees from `RappelzCore` (server) and `RappelzRust` (client), preserving their distinct Cargo members and histories. The initial combined folder remains on its unborn branch; do not reset or discard it. The root LICENSE was recovered from the existing upstream repository. Publish no ignored databases, credentials, original assets, converted MPQs, decompiler outputs or local runtime evidence.

Useful commands:

```powershell
cargo test --workspace --all-targets -- --test-threads=4
cargo build --workspace --release
cargo test -p rz-addon
target/release/rz-extractor.exe addons --out client-data/RappelzRust
target/release/RappelzRust.exe -data client-data/RappelzRust -demo
```

The exhaustive `rz-model::tests::all_files` remains intentionally ignored by default. It was not rerun this cycle; historical comprehensive counts must remain labeled historical until it is explicitly executed again.
