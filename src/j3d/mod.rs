//! The J3D scene format: the `bdl4` model tree and the `btk` texture animation.
//!
//! These read blocks out of a model file and rebuild the scene graph they describe, so they sit
//! below the game the way `libs/JSystem` sits below `src/d` in a decompilation project. Nothing
//! here may name anything from [`crate::sim`], [`crate::ui`] or [`crate::assets`], and neither
//! file here knows what a stage, an actor or a room is.
//!
//! A `bdl4` is a block container like an archive but with its own magic and its own block walk, so
//! [`bdl`] has its own reader rather than borrowing [`crate::archive`]'s: the block table is
//! name-addressed and the payloads are typed records, which is a different shape from an archive's
//! flat named ranges.

pub mod bdl;
pub mod btk;
