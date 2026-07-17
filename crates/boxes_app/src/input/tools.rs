//! Active tool mode and type palette.

use bevy::prelude::*;
use boxes_sim::{
    generator_period, make_aggregator, make_generator, make_transformer, Cell, Direction,
    ReduceMode,
};

/// Player tool mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveTool {
    #[default]
    Place,
    Erase,
    Inspect,
}

impl ActiveTool {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Place => "Place",
            Self::Erase => "Erase",
            Self::Inspect => "Inspect",
        }
    }
}

/// Factory cell preset for a palette slot.
#[derive(Clone, Copy, Debug)]
pub enum PalettePreset {
    Generator { period: u32 },
    Transformer { direction: Direction },
    Aggregator { mode: ReduceMode },
}

impl PalettePreset {
    #[must_use]
    pub fn to_cell(self) -> Cell {
        match self {
            Self::Generator { period } => make_generator(period, 0),
            Self::Transformer { direction } => make_transformer(direction, 0),
            Self::Aggregator { mode } => make_aggregator(mode, 0),
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Generator { .. } => "generator",
            Self::Transformer { .. } => "transformer",
            Self::Aggregator { .. } => "aggregator",
        }
    }
}

/// Tool and palette state for factory building.
#[derive(Resource)]
pub struct ToolState {
    pub active: ActiveTool,
    pub palette: [PalettePreset; 9],
    pub selected_slot: usize,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            active: ActiveTool::Place,
            palette: [
                PalettePreset::Generator {
                    period: generator_period::STANDARD,
                },
                PalettePreset::Generator {
                    period: generator_period::FAST,
                },
                PalettePreset::Generator {
                    period: generator_period::SLOW,
                },
                PalettePreset::Transformer {
                    direction: Direction::PosX,
                },
                PalettePreset::Transformer {
                    direction: Direction::PosZ,
                },
                PalettePreset::Aggregator {
                    mode: ReduceMode::Sum,
                },
                PalettePreset::Aggregator {
                    mode: ReduceMode::Max,
                },
                PalettePreset::Transformer {
                    direction: Direction::NegY,
                },
                PalettePreset::Transformer {
                    direction: Direction::PosY,
                },
            ],
            selected_slot: 0,
        }
    }
}

impl ToolState {
    #[must_use]
    pub fn selected_preset(&self) -> PalettePreset {
        self.palette[self.selected_slot]
    }
}

#[must_use]
pub fn palette_slot_label(slot: usize, preset: PalettePreset) -> String {
    let slot_num = slot + 1;
    match preset {
        PalettePreset::Generator { period } => {
            let seconds = period as f32 / 20.0;
            format!("{slot_num}. gen {seconds:.1}s")
        }
        PalettePreset::Transformer { direction } => {
            format!("{slot_num}. xform {}", direction_label(direction))
        }
        PalettePreset::Aggregator { mode } => {
            format!("{slot_num}. agg {}", reduce_mode_label(mode))
        }
    }
}

#[must_use]
pub fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::PosX => "+X",
        Direction::NegX => "-X",
        Direction::PosY => "+Y",
        Direction::NegY => "-Y",
        Direction::PosZ => "+Z",
        Direction::NegZ => "-Z",
    }
}

#[must_use]
pub fn reduce_mode_label(mode: ReduceMode) -> &'static str {
    match mode {
        ReduceMode::Sum => "sum",
        ReduceMode::Max => "max",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_presets_produce_typed_cells() {
        let tool = ToolState::default();
        let cell = tool.selected_preset().to_cell();
        assert!(!cell.is_empty());
    }

    #[test]
    fn active_tool_labels() {
        assert_eq!(ActiveTool::Place.label(), "Place");
        assert_eq!(ActiveTool::Erase.label(), "Erase");
        assert_eq!(ActiveTool::Inspect.label(), "Inspect");
    }
}
