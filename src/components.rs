use bevy::prelude::*;

/// The merge level of a shape (1 = Spark … 5 = Singularity for Phase-1).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShapeLevel(pub u32);

/// Position of an entity inside the merge grid.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GridPos {
    pub col: i32,
    pub row: i32,
}

/// Reference from a shape entity to its floating level-number label entity,
/// so we can despawn the label in O(1) without a position-based search.
#[derive(Component)]
pub struct LabelEntity(pub Entity);

/// Marks the label entity that displays a shape's level number.
#[derive(Component)]
pub struct ShapeLabel;

// ── UI marker components ─────────────────────────────────────────────────────

#[derive(Component)]
pub struct AuraDisplay;

#[derive(Component)]
pub struct RateDisplay;

#[derive(Component)]
pub struct TokenDisplay;
