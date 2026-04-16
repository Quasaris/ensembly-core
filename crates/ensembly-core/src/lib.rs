// Library target — exposes Core internals to ensembly-shell.
// The binary target (main.rs) keeps its own mod declarations for integration testing.
pub mod db;
pub mod ipc;
pub mod plugin_runtime;
