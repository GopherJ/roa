[![Stable Test](https://github.com/Hexilee/roa/workflows/Stable%20Test/badge.svg)](https://github.com/Hexilee/roa/actions)
[![codecov](https://codecov.io/gh/Hexilee/roa/branch/master/graph/badge.svg)](https://codecov.io/gh/Hexilee/roa)
[![Rust Docs](https://docs.rs/roa-tokio/badge.svg)](https://docs.rs/roa-tokio)
[![Crate version](https://img.shields.io/crates/v/roa-tokio.svg)](https://crates.io/crates/roa-tokio)
[![Download](https://img.shields.io/crates/d/roa-tokio.svg)](https://crates.io/crates/roa-tokio)
[![Version](https://img.shields.io/badge/rustc-1.40+-lightgray.svg)](https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Hexilee/roa/blob/master/LICENSE)

This crate provides tokio-based runtime and acceptor for roa.

```rust,no_run
use roa::http::StatusCode;
use roa::{App, Context};
use roa_tokio::{TcpIncoming, Exec};
use std::error::Error;

async fn end(_ctx: &mut Context) -> roa::Result {
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::with_exec((), Exec).end(end);
    let incoming = TcpIncoming::bind("127.0.0.1:0")?;
    println!("server is listening on {}", incoming.local_addr());
    app.accept(incoming).await?;
    Ok(())
}
```