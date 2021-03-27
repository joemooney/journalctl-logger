# Rust journalctl Logger

Provide a http openapi (REST) interface to provide logging
from journalctl to a file via ssh.

Since this uses **rocket** it requires Rust nightly:
`rustup override set nightly`

## Rationale
`netlogd` stopped writing to output messages over UDP but
ssh and `journalctl -f` was much more reliable.
