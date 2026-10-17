//! Reader, validator and resolver for the Multi-Agent Descriptor Language (MADL).
//!
//! ```no_run
//! let doc = madl::load(std::path::Path::new("team.madl.yaml")).unwrap();
//! let report = madl::validate(&doc);
//! assert!(report.is_ok());
//! let settings = madl::resolve(&madl::expand(&doc));
//! # let _ = settings;
//! ```
pub mod convert;
pub mod diagnostic;
pub mod duration;
pub mod expand;
pub mod graph;
pub mod load;
pub mod mermaid;
pub mod model;
pub mod resolve;
pub mod schema;
pub mod validate;

pub use diagnostic::{Diagnostic, Report, Severity};
pub use expand::expand;
pub use load::{LoadError, Options, from_str, load, load_with};
pub use model::Document;
pub use resolve::resolve;
pub use validate::validate;

/// The spec revisions this crate reads.
pub const REVISIONS: &[&str] = &["draft"];
