//! Compact per-cell storage.

/// Sentinel type id for empty cells.
pub const TYPE_EMPTY: u8 = 0;

/// Per-cell flag bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellFlags(pub u8);

impl CellFlags {
    pub const DIRTY: u8 = 1 << 0;

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn contains(self, bit: u8) -> bool {
        self.0 & bit != 0
    }

    pub const fn set(&mut self, bit: u8) {
        self.0 |= bit;
    }

    pub const fn clear(&mut self, bit: u8) {
        self.0 &= !bit;
    }
}

/// Fixed-size cell record (8 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Cell {
    pub type_id: u8,
    pub flags: u8,
    pub state: u16,
    pub reserved: u32,
}

impl Cell {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            type_id: TYPE_EMPTY,
            flags: 0,
            state: 0,
            reserved: 0,
        }
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.type_id == TYPE_EMPTY
    }

    #[must_use]
    pub fn flags(self) -> CellFlags {
        CellFlags(self.flags)
    }

    pub fn set_flags(&mut self, flags: CellFlags) {
        self.flags = flags.0;
    }
}
