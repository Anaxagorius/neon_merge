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

// ── Rebirth & Paragon UI markers ──────────────────────────────────────────────

/// Marker on the top-HUD text that shows rebirth count + permanent multiplier.
#[derive(Component)]
pub struct RebirthDisplay;

/// Identifies which Paragon upgrade a button / label belongs to.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParagonKind {
    AuraBoost,
    TokenRegen,
}

/// Marks the Rebirth action button.
#[derive(Component)]
pub struct RebirthButton;

/// Marks the text label inside the Rebirth button so it can be updated to show
/// availability, expected PP gain, and the permanent multiplier preview.
#[derive(Component)]
pub struct RebirthButtonLabel;

/// Marks a Paragon upgrade button.
#[derive(Component)]
pub struct ParagonButton(pub ParagonKind);

/// Marks the text child of a Paragon button so its label can be updated.
#[derive(Component)]
pub struct ParagonLabel(pub ParagonKind);

// ── Upgrade system ───────────────────────────────────────────────────────────

/// Identifies which upgrade a button or label belongs to.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpgradeKind {
    AuraMultiplier,
    TokenCapacity,
    MergeSpeed,
}

/// Marks a UI button entity as an upgrade button.
#[derive(Component)]
pub struct UpgradeButton(pub UpgradeKind);

/// Marks the text child of an upgrade button so its label can be updated.
#[derive(Component)]
pub struct UpgradeLabel(pub UpgradeKind);
