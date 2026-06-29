# Timetable Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a KonomiTV-inspired timetable command and MCP tool that keep `programs search` as SearchPg search while exposing EnumPgInfoEx as timetable data.

**Architecture:** Keep `EdcbClient::enum_pg_info_ex()` as the raw CtrlCmd API. Add a `flows::get_timetable()` application flow that resolves target services, builds EDCB service/time filters, groups services into channels/subchannels, attaches best-effort reservation metadata, and returns a structured `TimeTable`. Add `edcb programs timetable` and MCP `get_timetable` as thin adapters over the flow.

**Tech Stack:** Rust 2024, Tokio, chrono, serde, schemars, rmcp, existing CtrlCmd codec and mock TCP fixtures.

## Global Constraints

- Keep `programs search` as the SearchPg/SearchKeyInfo search command.
- Add `programs timetable` for EnumPgInfoEx-backed program table retrieval.
- Do not introduce a persistent DB or KonomiTV channel database.
- Implement channel grouping and reservation annotation only from CtrlCmd data already available: `ServiceInfo`, `ServiceEventInfo`, and `ReserveData`.
- Keep the first version simple and deterministic.
- Update README and `.agents/skills/cli/SKILL.md`.

---

### Task 1: Timetable Types And Flow

**Files:**
- Modify: `src/types.rs`
- Modify: `src/flows.rs`
- Modify: `src/lib.rs`
- Modify: `tests/reservation.rs`

**Interfaces:**
- Produces: `ChannelType`, `TimeTableQuery`, `TimeTable`, `TimeTableDateRange`, `TimeTableChannel`, `TimeTableSubchannel`, `TimeTableProgram`, `TimeTableProgramReservation`, `ReservationStatus`, `RecordingAvailability`
- Produces: `flows::get_timetable(client: &EdcbClient, query: &TimeTableQuery) -> Result<TimeTable>`

- [x] Add failing tests that verify `get_timetable()` calls `enum_service`, `enum_pg_info_ex`, and `enum_reserve`, returns channels with programs, and attaches reservation metadata.
- [x] Add a second failing test that verifies subchannel grouping puts same-TS short-duration subchannels under the main channel.
- [x] Implement the timetable types.
- [x] Implement EDCB service/time filters using `datetime_to_file_time()`.
- [x] Implement service filtering by repeated `ServiceKey` and best-effort `ChannelType`.
- [x] Implement reservation matching by exact `(onid, tsid, sid, eid)` first, then service/time overlap fallback.
- [x] Implement same `(onid, tsid)` subchannel grouping with KonomiTV's 8-hour independence rule.
- [x] Run `cargo test --test reservation timetable`.
- [x] Add regression tests that keep reservation time-overlap fallback and long-subchannel independence during simplification.
- [x] Refactor timetable building into program indexing, channel building, and date-range calculation without dropping behavior.

### Task 2: CLI Timetable Command

**Files:**
- Modify: `src/cli.rs`
- Modify: `tests/cli.rs`

**Interfaces:**
- Consumes: `TimeTableQuery`
- Produces: `CliCommand::ProgramsTimetable(TimeTableQuery)`

- [x] Add failing CLI parse tests for `programs timetable --service ... --start-time ... --end-time ... --channel-type gr`.
- [x] Add `programs timetable` parsing.
- [x] Add human/plain formatting that flattens timetable programs as event-key, start, duration, service, title, reservation status.
- [x] Update help text.
- [x] Run `cargo test --test cli parses_program_timetable_command`.

### Task 3: MCP Tool And Docs

**Files:**
- Modify: `src/mcp.rs`
- Modify: `tests/mcp_server.rs`
- Modify: `README.md`
- Modify: `.agents/skills/cli/SKILL.md`

**Interfaces:**
- Produces: MCP tool `get_timetable`
- Produces: `GetTimetableParam`

- [x] Add failing MCP tests for param conversion and tool listing.
- [x] Add `get_timetable` tool with `start_time`, `end_time`, `channel_type`, and `services`.
- [x] Document CLI and MCP usage.
- [x] Run focused `cargo test --test mcp_server get_timetable_param_maps_to_query` and `cargo test --test mcp_server mcp_server_exposes_v1_tools`.

### Verification

- [x] `cargo fmt --check`
- [x] `cargo test`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `git diff --check`
- [x] Run an actual command against the known EDCB host when available:

```sh
target/debug/edcb --host 172.18.0.7 programs timetable --service 32736:32736:1024
```

Observed command during implementation:

```sh
cargo run --bin edcb -- --host 172.18.0.7 --timeout-seconds 30 programs timetable --service 32736:32736:1024 --start-time 2026-06-29T19:00:00+09:00 --end-time 2026-06-29T21:00:00+09:00 --plain
```

It returned the expected NHK program rows and attached reservation metadata such
as `459:Reserved:Full`.
