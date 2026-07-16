//! Cell type identifiers, encoding helpers, and factory defaults.

use crate::cell::Cell;

pub use crate::cell::TYPE_EMPTY;
/// Periodic output on phased ticks.
pub const TYPE_GENERATOR: u8 = 1;
/// Copies an input neighbor state when dirty.
pub const TYPE_TRANSFORMER: u8 = 2;
/// Reduces neighbor states when dirty.
pub const TYPE_AGGREGATOR: u8 = 3;

/// Generator profile periods from the P2 spec (20 Hz ticks).
pub mod generator_period {
    /// 0.5 s
    pub const FAST: u32 = 10;
    /// 1 s
    pub const STANDARD: u32 = 20;
    /// 5 s
    pub const SLOW: u32 = 100;
}

/// Aggregator reduce modes stored in `Cell::reserved` low byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ReduceMode {
    Sum = 0,
    Max = 1,
}

impl ReduceMode {
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Max,
            _ => Self::Sum,
        }
    }
}

/// Axis direction for transformer input (face neighbor index).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl Direction {
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value % 6 {
            1 => Self::NegX,
            2 => Self::PosY,
            3 => Self::NegY,
            4 => Self::PosZ,
            5 => Self::NegZ,
            _ => Self::PosX,
        }
    }
}

/// Static catalog metadata for the four v1 cell types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellTypeInfo {
    pub id: u8,
    pub name: &'static str,
    pub is_event_driven: bool,
    pub is_phase_driven: bool,
}

/// Registry of built-in cell types.
#[derive(Clone, Copy, Debug, Default)]
pub struct CellTypeRegistry;

impl CellTypeRegistry {
    pub const ALL: [CellTypeInfo; 4] = [
        CellTypeInfo {
            id: TYPE_EMPTY,
            name: "empty",
            is_event_driven: false,
            is_phase_driven: false,
        },
        CellTypeInfo {
            id: TYPE_GENERATOR,
            name: "generator",
            is_event_driven: false,
            is_phase_driven: true,
        },
        CellTypeInfo {
            id: TYPE_TRANSFORMER,
            name: "transformer",
            is_event_driven: true,
            is_phase_driven: false,
        },
        CellTypeInfo {
            id: TYPE_AGGREGATOR,
            name: "aggregator",
            is_event_driven: true,
            is_phase_driven: false,
        },
    ];

    #[must_use]
    pub fn get(type_id: u8) -> Option<&'static CellTypeInfo> {
        Self::ALL.iter().find(|info| info.id == type_id)
    }

    #[must_use]
    pub fn listens_to_neighbors(type_id: u8) -> bool {
        matches!(type_id, TYPE_TRANSFORMER | TYPE_AGGREGATOR)
    }
}

#[must_use]
pub fn make_generator(period_ticks: u32, output: u16) -> Cell {
    Cell {
        type_id: TYPE_GENERATOR,
        flags: 0,
        state: output,
        reserved: period_ticks,
    }
}

#[must_use]
pub fn make_transformer(direction: Direction, initial_state: u16) -> Cell {
    Cell {
        type_id: TYPE_TRANSFORMER,
        flags: 0,
        state: initial_state,
        reserved: u32::from(direction as u8),
    }
}

#[must_use]
pub fn make_aggregator(mode: ReduceMode, initial_state: u16) -> Cell {
    Cell {
        type_id: TYPE_AGGREGATOR,
        flags: 0,
        state: initial_state,
        reserved: u32::from(mode as u8),
    }
}

#[must_use]
pub fn generator_period(cell: Cell) -> u32 {
    cell.reserved.max(1)
}

#[must_use]
pub fn transformer_direction(cell: Cell) -> Direction {
    Direction::from_u8(cell.reserved as u8)
}

#[must_use]
pub fn aggregator_mode(cell: Cell) -> ReduceMode {
    ReduceMode::from_u8(cell.reserved as u8)
}

/// Returns true when a generator at `pos` should fire on simulation tick `tick`.
#[must_use]
pub fn generator_should_fire(tick: u64, pos: crate::coord::WorldPos, period_ticks: u32) -> bool {
    let period = period_ticks.max(1) as u64;
    let phase = u64::from(pos.phase());
    (tick + phase).is_multiple_of(period)
}
