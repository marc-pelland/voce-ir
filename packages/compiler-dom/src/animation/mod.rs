//! Animation compilation — tiered strategy for optimal output.
//!
//! - Tier 1 (CSS): transitions, springs via linear(), scroll-driven
//! - Tier 2 (WAAPI): choreographed sequences
//! - Tier 3 (rAF): interruptible springs, gesture-driven motion
//!
//! The spring solver runs at compile time in Rust — zero JS for spring animations.

pub mod spring;
