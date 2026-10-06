use bevy_ecs::prelude::*;

pub const TAGSO_ACTORS: [&str; 2] = ["TagSo", "TagMSo"];

/// `daTag_So_c` (`d_a_tag_so.cpp`). `getArg` (`:51-65`) is the actor's only reader of the
/// parameter and it decodes three fields, in this order:
///
/// | field     | decode                              | line |
/// |-----------|-------------------------------------|------|
/// | `mRndNum` | `GetParamBit(param, 0, 8)`          | `:56` |
/// | `mType`   | `GetParamBit(param, 16, 8)`         | `:58` |
/// | `mJumpRange` | `paramRadius == 0xFF ? 1600.0f : paramRadius * 100` where `paramRadius = GetParamBit(param, 8, 8)` | `:59-64` |
///
/// All three only feed omitted behaviour: `mRndNum`/`mJumpRange`/`mType` are consumed by
/// `daNpc_So_c` (`d_a_npc_so.cpp:240-243`, `:261-264` - not ported) and by `debugDraw`
/// (`:36-41`), which `_draw` (`:44-48`) only calls when the debug-only HIO flag `l_HIO.m05`
/// is set, and its constructor leaves that at `0` (`:24`). `_execute` (`:31-33`) is literally
/// `return TRUE` - an empty body, not a stub. `Mthd_IsDelete` (`:100-102`) is a bare TRUE, an
/// accepted delete request. So per RULE 7 nothing is stored: the decode has no reader here.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagSo;

impl TagSo {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_tagso(name: &str) -> bool {
    TAGSO_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&TagSo>) {}