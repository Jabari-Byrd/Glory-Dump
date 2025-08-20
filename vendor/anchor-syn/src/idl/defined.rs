use anyhow::{Result};
use proc_macro2::Span;
/// Minimal stub file just to satisfy anchor-lang build expecting IdlBuild traits.
/// We only provide the exact symbol anchor-lang calls: a function to obtain a crate path.
/// Our patched version avoids Span::source_file which older rustc in Solana toolchain lacks.
pub fn crate_path_from_span(span: Option<Span>) -> std::path::PathBuf {
    // Fallback: just use current directory instead of span.source_file().path()
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

// Additional minimal APIs could be added if anchor-lang tries to use them. For now this suffices
// to suppress the failing call site when we patch anchor-lang (if needed). However, since the
// error originates inside anchor-syn itself, replacing the crate with this simpler implementation
// sidesteps the missing method usage.
