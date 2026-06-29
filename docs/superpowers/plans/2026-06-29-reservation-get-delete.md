# Reservation Get/Delete Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class reservation detail retrieval and deletion to the Rust client, CLI, and MCP server.

**Architecture:** Keep `EdcbClient` as the raw CtrlCmd layer. Put application behavior that validates reservation existence in `flows`, then expose the same use cases through CLI commands and MCP tools.

**Tech Stack:** Rust, Tokio, rmcp, existing EDCB CtrlCmd codec and test fixtures.

## Global Constraints

- Prefer simple behavior: get by reserve ID and delete by reserve ID only.
- Require `--yes` for CLI mutation commands.
- Do not add reservation update or recording-setting editing in this change.
- Keep README command/tool lists current.

---

### Task 1: Reservation Get/Delete

**Files:**
- Modify: `src/client.rs`
- Modify: `src/flows.rs`
- Modify: `src/cli.rs`
- Modify: `src/mcp.rs`
- Modify: `tests/reservation.rs`
- Modify: `tests/cli.rs`
- Modify: `tests/mcp_server.rs`
- Modify: `README.md`

**Interfaces:**
- Consumes: `EdcbClient::get_reserve(reserve_id: i32) -> Result<ReserveData>`
- Produces: `EdcbClient::delete_reserve(reserve_id: i32) -> Result<()>`
- Produces: `EdcbClient::delete_reserves(reserve_ids: &[i32]) -> Result<()>`
- Produces: `flows::get_reservation(client: &EdcbClient, reserve_id: i32) -> Result<ReserveData>`
- Produces: `flows::delete_reservation(client: &EdcbClient, reserve_id: i32) -> Result<ReserveData>`
- Produces CLI commands `reserves get <reserve-id>` and `reserves delete <reserve-id> --yes`
- Produces MCP tools `get_reservation` and `delete_reservation`

- [x] **Step 1: Write failing tests**

Add tests that express the new behavior before production code exists:

```rust
// tests/cli.rs
#[test]
fn parses_reservation_get_and_delete_commands() {
    let get = CliAction::from_args_and_env(["edcb", "reserves", "get", "42"], empty_env())
        .expect("reservation get command should parse");
    let delete =
        CliAction::from_args_and_env(["edcb", "reserves", "delete", "42", "--yes"], empty_env())
            .expect("reservation delete command should parse");

    assert!(matches!(
        get,
        CliAction::Run(CliInvocation {
            command: CliCommand::ReserveGet(42),
            ..
        })
    ));
    assert!(matches!(
        delete,
        CliAction::Run(CliInvocation {
            command: CliCommand::ReserveDelete(42),
            ..
        })
    ));
}

#[test]
fn reserve_delete_requires_confirmation() {
    let error = CliAction::from_args_and_env(["edcb", "reserves", "delete", "42"], empty_env())
        .expect_err("reservation deletion should require --yes");
    assert_eq!(error.exit_code, 2);
}
```

```rust
// tests/reservation.rs
#[tokio::test]
async fn delete_reservation_fetches_existing_reserve_then_sends_delete() {
    let reserve = reserve_fixture_for_test();
    let (addr, server) =
        spawn_two_command_server(2012, encode_reserve_for_test(&reserve), 1014, Vec::new()).await;

    let mut client = EdcbClient::new(addr.ip().to_string(), addr.port());
    client.set_timeout(Duration::from_secs(1));

    let deleted = delete_reservation(&client, reserve.reserve_id)
        .await
        .expect("reservation delete should return the deleted reservation");
    let payloads = server
        .await
        .expect("mock EDCB server task should complete without panicking");

    assert_eq!(deleted.reserve_id, reserve.reserve_id);
    assert_eq!(&payloads[0][0..2], &5_u16.to_le_bytes());
    assert_eq!(&payloads[0][2..6], &reserve.reserve_id.to_le_bytes());
    assert_eq!(read_i32_at(&payloads[1], 0), 12);
    assert_eq!(read_i32_at(&payloads[1], 4), 1);
    assert_eq!(read_i32_at(&payloads[1], 8), reserve.reserve_id);
}
```

```rust
// tests/mcp_server.rs
assert!(tool_names.contains(&"get_reservation"));
assert!(tool_names.contains(&"delete_reservation"));
```

- [x] **Step 2: Run focused tests to verify they fail**

Run:

```bash
cargo test --test cli parses_reservation_get_and_delete_commands
cargo test --test reservation delete_reservation_fetches_existing_reserve_then_sends_delete
cargo test --test mcp_server mcp_server_exposes_v1_tools
```

Expected: failures due to missing enum variants, missing flow functions, missing client delete methods, and missing MCP tools.

- [x] **Step 3: Implement raw CtrlCmd delete**

Add `EdcbClient::delete_reserve` and `EdcbClient::delete_reserves` in `src/client.rs`:

```rust
pub async fn delete_reserve(&self, reserve_id: i32) -> Result<()> {
    self.delete_reserves(&[reserve_id]).await
}

pub async fn delete_reserves(&self, reserve_ids: &[i32]) -> Result<()> {
    self.send_cmd(CMD_EPG_SRV_DEL_RESERVE, |writer| {
        writer.write_vector(reserve_ids, |writer, reserve_id| writer.write_i32(*reserve_id))
    })
    .await?;
    Ok(())
}
```

- [x] **Step 4: Implement application flows**

Add functions in `src/flows.rs`:

```rust
pub async fn get_reservation(client: &EdcbClient, reserve_id: i32) -> Result<ReserveData> {
    client.get_reserve(reserve_id).await
}

pub async fn delete_reservation(client: &EdcbClient, reserve_id: i32) -> Result<ReserveData> {
    let reserve = get_reservation(client, reserve_id).await?;
    client.delete_reserve(reserve_id).await?;
    Ok(reserve)
}
```

- [x] **Step 5: Expose through CLI**

Add `ReserveGet(i32)` and `ReserveDelete(i32)` variants, parse:

```text
reserves get <reserve-id>
reserves delete <reserve-id> --yes
```

Render both with the existing reservation plain formatter.

- [x] **Step 6: Expose through MCP**

Add a `ReservationIdParam { reserve_id: i32 }`, and tools:

```rust
get_reservation -> flows::get_reservation(&client, params.reserve_id)
delete_reservation -> flows::delete_reservation(&client, params.reserve_id)
```

- [x] **Step 7: Update README**

Document:

```text
reserves get <reserve-id>
reserves delete <reserve-id> --yes
get_reservation
delete_reservation
```

Mark reserve delete mutation as supported in the To Do list wording.

- [x] **Step 8: Verify**

Run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Expected: all commands exit 0.
