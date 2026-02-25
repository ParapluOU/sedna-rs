# sedna-rs

An embedded Sedna XML Database for Rust.

## Overview

`sedna-rs` is a Rust library that embeds the [Sedna XML Database](http://sedna.org/), providing a complete XQuery-capable XML database that runs entirely within your Rust application. No external installation required - the database server binaries are compiled and embedded at build time.

> ### Looking for IT services?
> <img src="https://fromulo.com/codesociety.png" align="left" width="80" alt="CodeSociety">
>
> **[CodeSociety](https://codesocietyhub.com/)** is our consulting & contracting arm — specializing in
> **IT architecture**, **XML authoring systems**, **FontoXML integration**, and **TerminusDB consulting**.
> We build structured content platforms and data solutions that power digital publishing.
>
> **[Let's talk! &#8594;](https://codesocietyhub.com/contact.html)**

## Features

- **Fully Embedded**: Database server binaries are embedded in your application
- **Zero Configuration**: Server lifecycle is automatically managed
- **XQuery 1.0 Support**: Full XQuery implementation for powerful XML querying
- **ACID Transactions**: Complete transaction support with commit/rollback
- **Safe Rust API**: Type-safe wrappers around the C FFI layer
- **Multiple Instances**: Run multiple isolated database servers simultaneously
- **Unix/Mac Support**: Works on Linux and macOS (no Windows support)

## Requirements

### Build Time

- CMake 2.6 or higher
- C/C++ compiler (GCC or Clang)
- Make

### Runtime

- Unix-like operating system (Linux or macOS)
- No external dependencies

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sedna-rs = "0.1"
```

## Quick Start

```rust
use sedna_rs::{SednaServer, Result};

fn main() -> Result<()> {
    // Start an embedded Sedna server
    let server = SednaServer::new()?;

    // Connect to the default database
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;

    // Begin a transaction
    client.begin_transaction()?;

    // Load an XML document
    client.execute(r#"
        CREATE DOCUMENT "book" IN COLLECTION "library"
        <book>
            <title>The Rust Programming Language</title>
            <author>Steve Klabnik</author>
            <year>2018</year>
        </book>
    "#)?;

    // Commit the transaction
    client.commit_transaction()?;

    // Query the document
    let mut result = client.execute("doc('book')//title")?;

    while let Some(item) = result.next()? {
        println!("Title: {}", item);
    }

    Ok(())
}
```

## API Overview

### SednaServer

The `SednaServer` manages the lifecycle of an embedded Sedna database instance.

```rust
// Start server on default port (5050)
let server = SednaServer::new()?;

// Start server on specific port
let server = SednaServer::with_port(5060)?;

// Get the server's port
let port = server.port();

// Connect to a database
let client = server.connect("dbname", "username", "password")?;
```

The server automatically shuts down when dropped.

### SednaClient

The `SednaClient` provides methods for interacting with the database.

```rust
// Execute a query
let mut result = client.execute("doc('test')//item")?;

// Transaction management
client.begin_transaction()?;
client.commit_transaction()?;
client.rollback_transaction()?;
```

### QueryResult

The `QueryResult` allows iteration over query results.

```rust
// Iterate over results one by one
while let Some(item) = result.next()? {
    println!("{}", item);
}

// Collect all results into a vector
let all_results = result.collect_all()?;
```

## XQuery Examples

### Creating Documents

```rust
client.execute(r#"
    CREATE DOCUMENT "mydoc" IN COLLECTION "mycollection"
    <root>
        <item id="1">First</item>
        <item id="2">Second</item>
    </root>
"#)?;
```

### Querying

```rust
// Simple path expressions
client.execute("doc('mydoc')//item")?;

// With predicates
client.execute("doc('mydoc')//item[@id='1']")?;

// FLWOR expressions
client.execute(r#"
    for $item in doc('mydoc')//item
    where $item/@id > 1
    return $item
"#)?;
```

### Updating

```rust
// Replace a node
client.execute(r#"
    UPDATE replace $x in doc('mydoc')//item[@id='1']
    with <item id="1">Updated</item>
"#)?;

// Insert a node
client.execute(r#"
    UPDATE insert <item id="3">Third</item>
    into doc('mydoc')/root
"#)?;

// Delete a node
client.execute(r#"
    UPDATE delete $x in doc('mydoc')//item[@id='2']
"#)?;
```

## Architecture

`sedna-rs` works by:

1. **Build Time**: Compiling the Sedna C library (`libsedna`) and full Sedna server using CMake
2. **Embedding**: Including the compiled binaries in the Rust library using `include_bytes!()`
3. **Runtime**: Extracting binaries to a temporary directory when first needed
4. **Lifecycle**: Starting the `se_gov` (governor) process and creating databases as needed
5. **Cleanup**: Automatically terminating server processes and cleaning up temp files on drop

## Running Examples

```bash
cargo run --example quickstart
```

## Running Tests

```bash
cargo test
```

Note: Tests may take some time as each test starts its own embedded database server.

## Performance

The embedded server starts in approximately 100-200ms on modern hardware. Query performance is equivalent to standalone Sedna installations.

## Limitations

- Unix/Mac only (no Windows support)
- Each `SednaServer` instance runs isolated processes and uses system resources
- Port conflicts will occur if multiple servers use the same port

## Credits

This library embeds and wraps the [Sedna XML Database](http://sedna.org/), developed by the Institute for System Programming of the Russian Academy of Sciences (ISP RAS).

## License

Apache-2.0

## Contributing

Contributions welcome! Please ensure tests pass before submitting PRs.
