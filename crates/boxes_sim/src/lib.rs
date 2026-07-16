//! Simulation kernel for Boxes.
//!
//! This crate owns the grid model, cell types, and tick scheduler. It has no
//! Bevy dependency; `boxes_app` reads dirty chunks and drives rendering.

/// Placeholder until P1 implements the simulation core.
pub const STUB: &str = "boxes_sim stub — see docs/specs/P1-simulation-core.md";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_is_non_empty() {
        assert!(!STUB.is_empty());
    }
}
