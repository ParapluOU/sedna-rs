//! # sedna-rs
//!
//! An embedded Sedna XML Database for Rust.
//!
//! This library provides a complete embedded Sedna XML database that can be used
//! in Rust applications. The database server binaries are embedded in the library
//! and automatically managed.
//!
//! ## Quick Start
//!
//! ```no_run
//! use sedna_rs::{SednaServer, Result};
//!
//! fn main() -> Result<()> {
//!     // Start an embedded Sedna server
//!     let server = SednaServer::new()?;
//!
//!     // Connect to the default database
//!     let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;
//!
//!     // Execute a query
//!     let mut result = client.execute("doc('test')//title")?;
//!
//!     // Iterate over results
//!     while let Some(item) = result.next()? {
//!         println!("{}", item);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Embedded**: No need to install Sedna separately
//! - **Automatic lifecycle management**: Server starts and stops automatically
//! - **Safe API**: Rust-friendly wrappers around the C API
//! - **XQuery support**: Full XQuery 1.0 support
//! - **Transactions**: ACID transaction support
//!

mod binaries;
mod client;
mod config;
mod error;
mod ffi;
mod server;

pub use client::{QueryResult, SednaClient};
pub use error::{Result, SednaError};
pub use server::SednaServer;
