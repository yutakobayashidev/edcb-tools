# edcb-mcp

Rust client library for EDCB/EpgTimer CtrlCmd.

This crate currently provides a Tokio-based TCP client and binary codec for the
read-oriented CtrlCmd APIs used by EDCB integrations. The implementation is
ported from `xtne6f/edcb.py`, with KonomiTV's async usage used as a secondary
reference.

## Supported in v1

- TCP transport
- EDCB primitive, string, vector, struct, and `SYSTEMTIME` codec
- Service, EPG, reserve, recorded-file, tuner, plugin, auto-add, manual-add,
  and notify-status read APIs
- Utility parsers for `ChSet5.txt`, `LogoData.ini`, logo directory indexes, and
  program extended text

## To Do

- [ ] Unix domain socket transport
- [ ] Windows named pipe transport
- [ ] View app stream / SrvPipe stream helpers
- [ ] Reserve, recorded-file, auto-add, and manual-add mutation APIs
- [ ] MCP server surface

## Example

```rust
use std::time::Duration;

use edcb_mcp::EdcbClient;

#[tokio::main]
async fn main() -> edcb_mcp::Result<()> {
    let mut client = EdcbClient::new("127.0.0.1", 4510);
    client.set_timeout(Duration::from_secs(5));

    let services = client.enum_service().await?;
    for service in services {
        println!("{}: {}", service.sid, service.service_name);
    }

    Ok(())
}
```

## Development

Use the Nix dev shell through direnv, then run:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```
