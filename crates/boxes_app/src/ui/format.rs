//! Human-readable cell and HUD strings for factory UI panels.

use boxes_sim::{
    aggregator_mode, transformer_direction, Cell, CellTypeRegistry, TYPE_AGGREGATOR,
    TYPE_GENERATOR, TYPE_TRANSFORMER, WorldPos,
};

use crate::input::{direction_label, reduce_mode_label};

/// Format inspector body text for a picked cell (or empty prompt).
#[must_use]
pub fn format_inspector(pos: Option<WorldPos>, cell: Cell) -> String {
    let Some(pos) = pos else {
        return "Inspect a cell (RMB or inspect tool)".to_string();
    };

    let type_name = CellTypeRegistry::get(cell.type_id)
        .map(|info| info.name)
        .unwrap_or("unknown");

    let mut lines = vec![
        format!("pos: ({}, {}, {})", pos.x, pos.y, pos.z),
        format!("type: {type_name}"),
        format!("state: {}", cell.state),
    ];

    if cell.type_id == TYPE_GENERATOR {
        lines.push(format!("period_ticks: {}", cell.reserved.max(1)));
    } else if cell.type_id == TYPE_TRANSFORMER {
        lines.push(format!(
            "input: {}",
            direction_label(transformer_direction(cell))
        ));
    } else if cell.type_id == TYPE_AGGREGATOR {
        lines.push(format!(
            "mode: {}",
            reduce_mode_label(aggregator_mode(cell))
        ));
    }

    if cell.is_empty() {
        lines.push("empty cell".to_string());
    }

    lines.join("\n")
}

/// One-line throughput readout for the HUD.
#[must_use]
pub fn format_throughput(tick: u64, total_updates: u64, dirty_chunks: usize) -> String {
    format!("tick: {tick}  updates: {total_updates}  dirty chunks: {dirty_chunks}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxes_sim::{make_generator, make_transformer, Direction, generator_period};

    #[test]
    fn inspector_shows_generator_period() {
        let pos = WorldPos::new(1, 2, 3);
        let cell = make_generator(generator_period::STANDARD, 7);
        let text = format_inspector(Some(pos), cell);
        assert!(text.contains("(1, 2, 3)"));
        assert!(text.contains("generator"));
        assert!(text.contains("state: 7"));
        assert!(text.contains("period_ticks: 20"));
    }

    #[test]
    fn inspector_prompt_when_no_pick() {
        let text = format_inspector(None, Cell::empty());
        assert!(text.contains("Inspect a cell"));
    }

    #[test]
    fn inspector_shows_transformer_direction() {
        let cell = make_transformer(Direction::PosZ, 0);
        let text = format_inspector(Some(WorldPos::new(0, 0, 0)), cell);
        assert!(text.contains("input: +Z"));
    }
}
