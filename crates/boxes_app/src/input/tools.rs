//! Active tool mode, type palette, and per-view depth slice.

use bevy::prelude::*;
use boxes_sim::{
    generator_period, make_aggregator, make_generator, make_transformer, Cell, Direction,
    ReduceMode, WORLD_SIZE,
};

use crate::render::OrthoView;

/// Player tool mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveTool {
    #[default]
    Place,
    Erase,
    Inspect,
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

/// Depth slice along the axis perpendicular to each orthographic face.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewSlice {
    pub top_y: u16,
    pub front_z: u16,
    pub left_x: u16,
}

impl Default for ViewSlice {
    fn default() -> Self {
        let mid = (WORLD_SIZE / 2) as u16;
        Self {
            top_y: mid,
            front_z: mid,
            left_x: mid,
        }
    }
}

impl ViewSlice {
    #[must_use]
    pub fn depth(self, view: OrthoView) -> u16 {
        match view {
            OrthoView::Top => self.top_y,
            OrthoView::Front => self.front_z,
            OrthoView::Left => self.left_x,
        }
    }

    #[must_use]
    pub fn nudge(self, view: OrthoView, delta: i16) -> u16 {
        let current = self.depth(view) as i32;
        let next = (current + i32::from(delta)).clamp(0, i32::from(WORLD_SIZE as u16 - 1));
        next as u16
    }

    pub fn set_depth(&mut self, view: OrthoView, depth: u16) {
        let depth = depth.min(WORLD_SIZE as u16 - 1);
        match view {
            OrthoView::Top => self.top_y = depth,
            OrthoView::Front => self.front_z = depth,
            OrthoView::Left => self.left_x = depth,
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

/// Last cell inspected via RMB or inspect tool.
#[derive(Resource, Default, Clone, Debug)]
pub struct InspectedCell {
    pub pos: Option<boxes_sim::WorldPos>,
    pub cell: Cell,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slice_is_world_midpoint() {
        let slice = ViewSlice::default();
        assert_eq!(slice.depth(OrthoView::Top), 250);
    }

    #[test]
    fn nudge_clamps_to_world_bounds() {
        let slice = ViewSlice::default();
        assert_eq!(slice.nudge(OrthoView::Top, 10_000), WORLD_SIZE as u16 - 1);
        assert_eq!(slice.nudge(OrthoView::Top, -10_000), 0);
    }

    #[test]
    fn palette_presets_produce_typed_cells() {
        let tool = ToolState::default();
        let cell = tool.selected_preset().to_cell();
        assert!(!cell.is_empty());
    }
}
