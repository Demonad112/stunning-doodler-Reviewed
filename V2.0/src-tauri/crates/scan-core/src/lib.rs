//! Folder scanning and size-based folder comparison.
//!
//! [`scan`] walks a folder into a [`Tree`] whose folder rows carry recursive totals. [`compare`]
//! matches two trees by relative path and produces a [`DiffTree`] that the UI expands lazily.

mod diff;
mod scan;

pub use diff::{compare, DiffNode, DiffRow, DiffSide, DiffStatus, DiffSummary, DiffTree};
pub use scan::{scan, CancelToken, Kind, Node, NodeId, ScanError, ScanProgress, ScanState, Tree};
