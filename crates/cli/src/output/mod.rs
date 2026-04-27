//! Standardized CLI output module
//!
//! This module provides the output contract infrastructure for all CLI commands.
//! It defines the envelope structures that wrap command outputs in a consistent,
//! parseable format across YAML, JSON, TOML, and human-readable table formats.

pub mod envelope;
pub mod formatter;
pub mod help;

pub use envelope::{
    ActiveContextState, CommandOutput, ErrorDetail, NextStep, OutputEnvelope, Status,
};
pub use formatter::{OutputFormat, format_help_document, format_output};
pub use help::{HelpDocument, StructuredHelpFormat, format_help_human};
