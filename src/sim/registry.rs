use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;
use glam::{Mat4, Vec3};

use super::actor::Actor;
use super::jpa::Clip;

pub const ERROR_PROC_ID: u32 = 0xFFFF_FFFF;
pub const RESERVED_PROC_ID: u32 = 0xFFFF_FFFE;
pub const FIRST_PROC_ID: u32 = 1;
pub const CLASS_ID_BASE: u32 = 0x0913_0000;
pub const BASE_CLASS_ID: u32 = 0x0913_0001;
pub const ACTOR_CLASS_ID: u32 = 0x0913_0002;
pub const NO_CLASS_ID: u32 = 0;
pub const UNKNOWN_NAME: &str = "UNKOWN";
pub const NO_SET_ID: u16 = 0xFFFF;
pub const ROOT_NODE_LISTS: usize = 10;

pub const ROOT_LAYER: u32 = 0;
pub const CURRENT_LAYER: u32 = 0xFFFF_FFFD;
pub const SPECIAL_LAYER: u32 = 0xFFFF_FFFE;
pub const NO_LAYER: u32 = 0xFFFF_FFFF;

pub const INIT_FIRST: u8 = 0;
pub const INIT_CREATING: u8 = 1;
pub const INIT_EXECUTING: u8 = 2;
pub const INIT_DELETING: u8 = 3;

pub const PHASE_INIT: i32 = 0;
pub const PHASE_LOADING: i32 = 1;
pub const PHASE_NEXT: i32 = 2;
pub const PHASE_STOP: i32 = 3;
pub const PHASE_COMPLEATE: i32 = 4;
pub const PHASE_ERROR: i32 = 5;

pub const PASS_DELETE: u8 = 0;
pub const PASS_PRIORITY: u8 = 1;
pub const PASS_CREATE: u8 = 2;
pub const PASS_EXECUTE: u8 = 3;
pub const PASS_DRAW: u8 = 4;
pub const MANAGEMENT_PASSES: [u8; 5] = [
    PASS_DELETE,
    PASS_PRIORITY,
    PASS_CREATE,
    PASS_EXECUTE,
    PASS_DRAW,
];

pub const STATUS_SHOW_MAP: u32 = 0x0000_0020;
pub const STATUS_UNK40: u32 = 0x0000_0040;
pub const STATUS_NO_CULL_EXEC: u32 = 0x0000_0080;
pub const STATUS_CULL: u32 = 0x0000_0100;
pub const STATUS_FREEZE: u32 = 0x0000_0400;
pub const STATUS_UNK800: u32 = 0x0000_0800;
pub const STATUS_UNK1000: u32 = 0x0000_1000;
pub const STATUS_CARRY: u32 = 0x0000_2000;
pub const STATUS_UNK4000: u32 = 0x0000_4000;
pub const STATUS_FORCE_MOVE: u32 = 0x0000_8000;
pub const STATUS_UNK10000: u32 = 0x0001_0000;
pub const STATUS_NO_PAUSE: u32 = 0x0002_0000;
pub const STATUS_UNK40000: u32 = 0x0004_0000;
pub const STATUS_UNK80000: u32 = 0x0008_0000;
pub const STATUS_HOOK_CARRY: u32 = 0x0010_0000;
pub const STATUS_UNK200000: u32 = 0x0020_0000;
pub const STATUS_NO_DRAW: u32 = 0x0100_0000;
pub const STATUS_UNK2000000: u32 = 0x0200_0000;
pub const STATUS_BOSS: u32 = 0x0400_0000;
pub const STATUS_UNK8000000: u32 = 0x0800_0000;
pub const STATUS_UNK10000000: u32 = 0x1000_0000;
pub const STATUS_MAP_ICON: u32 = 0x0000_001F;
pub const STATUS_MAP_ALL: u32 = 0x0000_003F;

pub const CONDITION_NO_EXEC: u32 = 0x02;
pub const CONDITION_NO_DRAW: u32 = 0x04;
pub const CONDITION_INIT: u32 = 0x08;

pub const GROUP_ACTOR: u8 = 0x0;
pub const GROUP_PLAYER: u8 = 0x1;
pub const GROUP_ENEMY: u8 = 0x2;
pub const GROUP_ENV: u8 = 0x3;
pub const GROUP_NPC: u8 = 0x4;

pub const PRIORITY_CURRENT: u16 = 0xFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorEntry {
    pub entity: Entity,
    pub proc_id: u32,
    pub class_id: u32,
    pub group: u8,
    pub status: u32,
    pub condition: u32,
    pub grab_actor_id: u32,
    pub parent_actor_id: u32,
    pub argument: i8,
    pub init_state: u8,
    pub create_result: i32,
    pub layer: u32,
    pub list_id: u16,
    pub priority: u16,
}

impl ActorEntry {
    pub fn new(entity: Entity, proc_id: u32, class_id: u32, group: u8, argument: i8) -> Self {
        Self {
            entity,
            proc_id,
            class_id,
            group,
            status: 0,
            condition: 0,
            grab_actor_id: ERROR_PROC_ID,
            parent_actor_id: ERROR_PROC_ID,
            argument,
            init_state: INIT_FIRST,
            create_result: PHASE_INIT,
            layer: CURRENT_LAYER,
            list_id: 0,
            priority: PRIORITY_CURRENT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingCreate {
    pub proc_id: u32,
    pub group: u8,
    pub argument: i8,
    pub parent_actor_id: u32,
    pub status: u32,
    pub condition: u32,
}

impl PendingCreate {
    pub fn new(proc_id: u32) -> Self {
        Self {
            proc_id,
            group: GROUP_ACTOR,
            argument: -1,
            parent_actor_id: ERROR_PROC_ID,
            status: 0,
            condition: 0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ActorRegistry {
    entries: Vec<Option<ActorEntry>>,
    free: Vec<usize>,
    pending: Vec<PendingCreate>,
    next_proc_id: u32,
    next_class_id: u32,
    next_layer: u32,
    root_layer: u32,
    base_class_id: u32,
    actor_class_id: u32,
    stop_status: u32,
    layer_creating: i16,
    layer_deleting: i16,
}

impl Default for ActorRegistry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
            pending: Vec::new(),
            next_proc_id: FIRST_PROC_ID,
            next_class_id: CLASS_ID_BASE,
            next_layer: ROOT_LAYER,
            root_layer: ROOT_LAYER,
            base_class_id: NO_CLASS_ID,
            actor_class_id: NO_CLASS_ID,
            stop_status: 0,
            layer_creating: 0,
            layer_deleting: 0,
        }
    }
}

impl ActorRegistry {
    pub fn init(&mut self) {
        self.base_class_id = self.make_class_id();
        self.actor_class_id = self.make_class_id();
        self.root_layer = self.make_layer();
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.free.clear();
        self.pending.clear();
        self.next_proc_id = FIRST_PROC_ID;
        self.next_class_id = CLASS_ID_BASE;
        self.next_layer = ROOT_LAYER;
        self.root_layer = ROOT_LAYER;
        self.base_class_id = NO_CLASS_ID;
        self.actor_class_id = NO_CLASS_ID;
        self.stop_status = 0;
        self.layer_creating = 0;
        self.layer_deleting = 0;
        self.init();
    }

    pub fn make_proc_id(&mut self) -> u32 {
        let id = self.next_proc_id;
        self.next_proc_id = self.next_proc_id.wrapping_add(1);
        id
    }

    pub fn make_class_id(&mut self) -> u32 {
        self.next_class_id = self.next_class_id.wrapping_add(1);
        self.next_class_id
    }

    pub fn make_layer(&mut self) -> u32 {
        let id = self.next_layer;
        self.next_layer = self.next_layer.wrapping_add(1);
        id
    }

    pub fn base_class_id(&self) -> u32 {
        self.base_class_id
    }

    pub fn actor_class_id(&self) -> u32 {
        self.actor_class_id
    }

    pub fn root_layer(&self) -> u32 {
        self.root_layer
    }

    pub fn is_root_layer(&self, layer: u32) -> bool {
        layer == self.root_layer || layer == ROOT_LAYER
    }

    pub fn stop_status(&self) -> u32 {
        self.stop_status
    }

    pub fn set_stop_status(&mut self, status: u32) {
        self.stop_status = status;
    }

    pub fn count(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &ActorEntry> {
        self.entries.iter().flatten()
    }

    pub fn reserve(&mut self, group: u8, argument: i8, parent_actor_id: u32) -> u32 {
        let mut pending = PendingCreate::new(self.make_proc_id());
        pending.group = group;
        pending.argument = argument;
        pending.parent_actor_id = parent_actor_id;
        let id = pending.proc_id;
        self.pending.push(pending);
        self.layer_creating += 1;
        id
    }

    pub fn cancel_create(&mut self, proc_id: u32) -> bool {
        let Some(index) = self.pending.iter().position(|slot| slot.proc_id == proc_id) else {
            return false;
        };
        self.pending.remove(index);
        self.created_message();
        true
    }

    pub fn finish_create(&mut self, entity: Entity, proc_id: u32) -> bool {
        let Some(index) = self.pending.iter().position(|slot| slot.proc_id == proc_id) else {
            return false;
        };
        let pending = self.pending.remove(index);
        let mut entry = ActorEntry::new(
            entity,
            pending.proc_id,
            self.actor_class_id,
            pending.group,
            pending.argument,
        );
        entry.status = pending.status;
        entry.condition = pending.condition;
        entry.parent_actor_id = pending.parent_actor_id;
        entry.init_state = INIT_EXECUTING;
        entry.create_result = PHASE_COMPLEATE;
        self.insert(entry);
        self.created_message();
        true
    }

    pub fn register(&mut self, entity: Entity, group: u8, argument: i8) -> u32 {
        let proc_id = self.reserve(group, argument, ERROR_PROC_ID);
        if !self.finish_create(entity, proc_id) {
            return ERROR_PROC_ID;
        }
        proc_id
    }

    fn insert(&mut self, entry: ActorEntry) {
        match self.free.pop() {
            Some(slot) => self.entries[slot] = Some(entry),
            None => self.entries.push(Some(entry)),
        }
    }

    pub fn remove(&mut self, proc_id: u32) -> Option<ActorEntry> {
        let index = self
            .entries
            .iter()
            .position(|slot| slot.is_some_and(|entry| entry.proc_id == proc_id))?;
        let entry = self.entries[index].take();
        if entry.is_some() {
            self.free.push(index);
        }
        entry
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<ActorEntry> {
        let index = self
            .entries
            .iter()
            .position(|slot| slot.is_some_and(|entry| entry.entity == entity))?;
        let entry = self.entries[index].take();
        if entry.is_some() {
            self.free.push(index);
        }
        entry
    }

    pub fn entry(&self, proc_id: u32) -> Option<&ActorEntry> {
        self.entries
            .iter()
            .flatten()
            .find(|entry| entry.proc_id == proc_id)
    }

    pub fn entry_mut(&mut self, proc_id: u32) -> Option<&mut ActorEntry> {
        self.entries
            .iter_mut()
            .flatten()
            .find(|entry| entry.proc_id == proc_id)
    }

    pub fn entity(&self, proc_id: u32) -> Option<Entity> {
        self.entry(proc_id).map(|entry| entry.entity)
    }

    pub fn entry_of(&self, entity: Entity) -> Option<&ActorEntry> {
        self.entries
            .iter()
            .flatten()
            .find(|entry| entry.entity == entity)
    }

    pub fn proc_id(&self, entity: Entity) -> Option<u32> {
        self.entry_of(entity).map(|entry| entry.proc_id)
    }

    pub fn is_registered(&self, entity: Entity) -> bool {
        self.entry_of(entity).is_some()
    }

    pub fn is_creating(&self, proc_id: u32) -> bool {
        self.pending.iter().any(|slot| slot.proc_id == proc_id)
    }

    pub fn is_first_creating(&self, proc_id: u32) -> bool {
        self.entry(proc_id)
            .is_some_and(|entry| entry.init_state == INIT_FIRST)
    }

    pub fn is_deleting(&self, proc_id: u32) -> bool {
        self.entry(proc_id)
            .is_some_and(|entry| entry.init_state == INIT_DELETING)
    }

    pub fn is_searchable(&self, proc_id: u32) -> bool {
        self.entry(proc_id)
            .is_some_and(|entry| entry.init_state == INIT_EXECUTING)
    }

    pub fn request_delete(&mut self, proc_id: u32) -> bool {
        if self.is_creating(proc_id) || self.is_deleting(proc_id) {
            return false;
        }
        let Some(entry) = self.entry_mut(proc_id) else {
            return true;
        };
        entry.init_state = INIT_DELETING;
        entry.create_result = PHASE_STOP;
        self.layer_deleting += 1;
        true
    }

    pub fn creating_message(&mut self) {
        self.layer_creating += 1;
    }

    pub fn created_message(&mut self) {
        if self.layer_creating > 0 {
            self.layer_creating -= 1;
        }
    }

    pub fn deleting_message(&mut self) {
        self.layer_deleting += 1;
    }

    pub fn deleted_message(&mut self) {
        if self.layer_deleting > 0 {
            self.layer_deleting -= 1;
        }
    }

    pub fn is_creating_room(&self) -> bool {
        self.layer_creating > 0
    }

    pub fn is_deleting_room(&self) -> bool {
        self.layer_deleting > 0
    }

    pub fn set_group(&mut self, proc_id: u32, group: u8) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.group = group;
        }
    }

    pub fn group(&self, proc_id: u32) -> Option<u8> {
        self.entry(proc_id).map(|entry| entry.group)
    }

    pub fn chk_status(&self, proc_id: u32, flag: u32) -> u32 {
        self.entry(proc_id).map_or(0, |entry| entry.status & flag)
    }

    pub fn on_status(&mut self, proc_id: u32, flag: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.status |= flag;
        }
    }

    pub fn off_status(&mut self, proc_id: u32, flag: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.status &= !flag;
        }
    }

    pub fn set_status(&mut self, proc_id: u32, status: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.status = status;
        }
    }

    pub fn status(&self, proc_id: u32) -> Option<u32> {
        self.entry(proc_id).map(|entry| entry.status)
    }

    pub fn chk_status_map(&self, proc_id: u32, _flag: u32) -> bool {
        let Some(status) = self.status(proc_id) else {
            return false;
        };
        status & STATUS_SHOW_MAP != 0 && status & STATUS_MAP_ICON == 0
    }

    pub fn set_status_map(&mut self, proc_id: u32, flag: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.status = (entry.status & !STATUS_MAP_ALL) | STATUS_SHOW_MAP | flag;
        }
    }

    pub fn clear_status_map(&mut self, proc_id: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.status &= !STATUS_MAP_ALL;
        }
    }

    pub fn chk_carry_now(&self, proc_id: u32) -> u32 {
        self.chk_status(proc_id, STATUS_CARRY)
    }

    pub fn chk_hook_carry_now(&self, proc_id: u32) -> bool {
        self.chk_status(proc_id, STATUS_HOOK_CARRY) != 0
    }

    pub fn set_hook_carry_now(&mut self, proc_id: u32) {
        self.on_status(proc_id, STATUS_HOOK_CARRY);
    }

    pub fn cancel_hook_carry_now(&mut self, proc_id: u32) {
        self.off_status(proc_id, STATUS_HOOK_CARRY);
    }

    pub fn chk_condition(&self, proc_id: u32, flag: u32) -> u32 {
        self.entry(proc_id)
            .map_or(0, |entry| entry.condition & flag)
    }

    /// `fopAc_Draw`'s predicate (`f_op_actor.cpp:129-140`): the actor is drawn unless the
    /// no-draw status bit is up, or the stop status is and a `CULL` actor's box is outside the
    /// clipper. `dComIfGp_event_moveApproval` (`moveApproval`, whose `FORCE` arm bypasses both
    /// tests) and a non-zero `fopAcM_GetCullSizeFar` have no port counterpart; every port
    /// actor's is 0, retail's own default, which skips the far branch
    /// (`f_op_actor_mng.cpp:602-648`). A missing clip or cull box leaves the actor drawn,
    /// because the port cannot evaluate a test it has no inputs for.
    ///
    /// `cull` is the actor's `fopAc_CULLBOX_CUSTOM_e` box and `model` the matrix `fopAcM_SetMtx`
    /// publishes (`f_op_actor_mng.cpp:284-286`), which for Ba1 is the morf's base TRM written by
    /// `setMtx` (`d_a_npc_ba1.cpp:288-291`).
    pub fn draw_gate(
        &self,
        entry: &ActorEntry,
        clip: Option<&Clip>,
        model: Mat4,
        cull: Option<([f32; 3], [f32; 3])>,
    ) -> bool {
        if entry.status & STATUS_NO_DRAW != 0 {
            return false;
        }
        if entry.status & self.stop_status != 0 {
            return false;
        }
        if entry.status & STATUS_CULL != 0
            && let (Some(clip), Some((min, max))) = (clip, cull)
            && clip.clip_box(model, Vec3::from(min), Vec3::from(max))
        {
            return false;
        }
        true
    }

    /// `fopAc_Execute`'s `NOCULLEXEC` clause (`f_op_actor.cpp:180-193`): a `NOCULLEXEC` actor
    /// whose last draw was rejected (`fopAcCnd_NODRAW_e`, written by [`ActorRegistry::draw_gate`]'s
    /// caller) skips its execute. `moveApproval` is the other half of retail's condition and has
    /// no port counterpart; `stopStatus` is 0 unless a caller sets one.
    pub fn execute_gate(&self, entry: &ActorEntry) -> bool {
        entry.status & STATUS_NO_CULL_EXEC == 0 || entry.condition & CONDITION_NO_DRAW == 0
    }

    /// The bookkeeping `fopAc_Draw` performs around `_draw` (`f_op_actor.cpp:141-150`):
    /// `fopAcM_OnCondition(NODRAW)` when the gate rejected the actor and the matching
    /// `OffCondition` when it ran, then `fopAcM_OffStatus(NODRAW)`.
    pub fn set_draw_condition(&mut self, proc_id: u32, drawn: bool) {
        if drawn {
            self.off_condition(proc_id, CONDITION_NO_DRAW);
        } else {
            self.on_condition(proc_id, CONDITION_NO_DRAW);
        }
        self.off_status(proc_id, STATUS_NO_DRAW);
    }

    pub fn on_condition(&mut self, proc_id: u32, flag: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.condition |= flag;
        }
    }

    pub fn off_condition(&mut self, proc_id: u32, flag: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.condition &= !flag;
        }
    }

    pub fn construct(&mut self, proc_id: u32) -> bool {
        if self.chk_condition(proc_id, CONDITION_INIT) != 0 {
            return false;
        }
        self.on_condition(proc_id, CONDITION_INIT);
        true
    }

    pub fn grab_actor_id(&self, proc_id: u32) -> u32 {
        self.entry(proc_id)
            .map_or(ERROR_PROC_ID, |entry| entry.grab_actor_id)
    }

    pub fn set_grab_actor(&mut self, proc_id: u32, grab_actor_id: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.grab_actor_id = grab_actor_id;
        }
    }

    pub fn clear_grab_actor(&mut self, proc_id: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.grab_actor_id = ERROR_PROC_ID;
        }
    }

    pub fn link_id(&self, proc_id: u32) -> u32 {
        self.entry(proc_id)
            .map_or(ERROR_PROC_ID, |entry| entry.parent_actor_id)
    }

    pub fn set_link_id(&mut self, proc_id: u32, parent_actor_id: u32) {
        if let Some(entry) = self.entry_mut(proc_id) {
            entry.parent_actor_id = parent_actor_id;
        }
    }

    pub fn set_layer(&mut self, proc_id: u32, layer: u32) -> bool {
        let Some(entry) = self.entry_mut(proc_id) else {
            return false;
        };
        if entry.init_state == INIT_DELETING || entry.init_state == INIT_FIRST {
            return false;
        }
        entry.layer = layer;
        true
    }

    pub fn set_priority(&mut self, proc_id: u32, priority: u16) -> bool {
        let Some(entry) = self.entry_mut(proc_id) else {
            return false;
        };
        if entry.init_state == INIT_DELETING {
            return false;
        }
        entry.priority = priority;
        true
    }
}

pub fn is_error_id(proc_id: u32) -> bool {
    proc_id == ERROR_PROC_ID
}

pub fn is_reserved_id(proc_id: u32) -> bool {
    proc_id.wrapping_add(2) <= 1
}

pub fn is_just_of_class(type_a: u32, type_b: u32) -> bool {
    type_a == type_b
}

pub fn is_actor_class(registry: &ActorRegistry, class_id: u32) -> bool {
    is_just_of_class(registry.actor_class_id(), class_id)
}

pub fn param_bit(param: u32, shift: u8, bit: u8) -> u32 {
    (param >> shift) & ((1 << bit) - 1)
}

pub fn actor_distance(from: Vec3, to: Vec3) -> f32 {
    (to - from).length()
}

pub fn actor_distance2(from: Vec3, to: Vec3) -> f32 {
    (to - from).length_squared()
}

pub fn actor_distance_xz(from: Vec3, to: Vec3) -> f32 {
    let delta = to - from;
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

pub fn actor_distance_xz2(from: Vec3, to: Vec3) -> f32 {
    let delta = to - from;
    delta.x * delta.x + delta.z * delta.z
}

pub fn actor_distance_y(from: Vec3, to: Vec3) -> f32 {
    to.y - from.y
}

pub fn actor_angle_y(from: Vec3, to: Vec3) -> i16 {
    super::walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

pub fn seen_actor_angle_y(from: Vec3, shape_y: i16, to: Vec3) -> i32 {
    i32::from(actor_angle_y(from, to).wrapping_sub(shape_y)).abs()
}

pub fn name_or_unknown(name: &str) -> &str {
    if name.is_empty() { UNKNOWN_NAME } else { name }
}

pub fn search_by_id<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    proc_id: u32,
) -> Option<Entity> {
    let entity = registry.entity(proc_id)?;
    actors.get(entity).ok().map(|(entity, _)| entity)
}

pub fn search_by_id_ex<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    proc_id: u32,
) -> (Option<Entity>, bool) {
    if is_reserved_id(proc_id) || registry.is_creating(proc_id) {
        return (None, true);
    }
    match search_by_id(registry, actors, proc_id) {
        Some(entity) => (Some(entity), true),
        None => (None, false),
    }
}

pub fn is_executing<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    proc_id: u32,
) -> bool {
    if is_reserved_id(proc_id) {
        return false;
    }
    search_by_id(registry, actors, proc_id).is_some()
}

pub fn search_by_name<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    name: &str,
) -> Option<Entity> {
    for entry in registry.iter() {
        if let Ok((entity, actor)) = actors.get(entry.entity) {
            if actor.name == name {
                return Some(entity);
            }
        }
    }
    None
}

pub fn search_by_name_ex<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    name: &str,
) -> (Option<Entity>, bool) {
    let Some(entity) = search_by_name(registry, actors, name) else {
        return (None, false);
    };
    if let Some(proc_id) = registry.proc_id(entity) {
        if registry.is_creating(proc_id) {
            return (None, true);
        }
    }
    (Some(entity), true)
}

pub fn search_from_name<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    name: &str,
    param_mask: u32,
    parameter: u32,
) -> Option<Entity> {
    for entry in registry.iter() {
        let Ok((entity, actor)) = actors.get(entry.entity) else {
            continue;
        };
        if actor.name != name {
            continue;
        }
        if param_mask == 0 || parameter == (actor.parameters & param_mask) {
            return Some(entity);
        }
    }
    None
}

pub fn search_by_class<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    class_id: u32,
) -> Option<Entity> {
    for entry in registry.iter() {
        if !is_just_of_class(entry.class_id, class_id) {
            continue;
        }
        if let Ok((entity, _)) = actors.get(entry.entity) {
            return Some(entity);
        }
    }
    None
}

pub fn search_by_group<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    group: u8,
) -> Option<Entity> {
    for entry in registry.iter() {
        if entry.group != group {
            continue;
        }
        if let Ok((entity, _)) = actors.get(entry.entity) {
            return Some(entity);
        }
    }
    None
}

pub fn search_in_room_by_group<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    group: u8,
    room_no: i8,
) -> Option<Entity> {
    for entry in registry.iter() {
        if entry.group != group {
            continue;
        }
        if let Ok((entity, actor)) = actors.get(entry.entity) {
            if actor.room == room_no {
                return Some(entity);
            }
        }
    }
    None
}

pub fn search_room_enemy<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    room_no: i8,
    grab_actor_id: u32,
) -> Option<Entity> {
    if let Some(entity) = search_by_id(registry, actors, grab_actor_id) {
        if registry.group(grab_actor_id) == Some(GROUP_ENEMY) {
            return Some(entity);
        }
    }
    search_in_room_by_group(registry, actors, GROUP_ENEMY, room_no)
}

fn nearest_match<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    center: Vec3,
    radius: f32,
    mut accept: impl FnMut(&ActorEntry, &Actor, f32) -> bool,
) -> Option<(Entity, Vec3)> {
    let mut best: Option<(f32, Entity, Vec3)> = None;
    for entry in registry.iter() {
        let Ok((entity, actor)) = actors.get(entry.entity) else {
            continue;
        };
        let offset = actor.pos - center;
        let dist = offset.length();
        if dist >= radius {
            continue;
        }
        if !accept(entry, actor, dist) {
            continue;
        }
        let replace = best.is_none_or(|(current, _, _)| dist < current);
        if replace {
            best = Some((dist, entity, offset));
        }
    }
    best.map(|(_, entity, offset)| (entity, offset))
}

pub fn search_nearest_by_name<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    name: &str,
    center: Vec3,
    radius: f32,
) -> Option<(Entity, Vec3)> {
    nearest_match(registry, actors, center, radius, |_, actor, _| {
        actor.name == name
    })
}

pub fn search_nearest_by_class<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    class_id: u32,
    center: Vec3,
    radius: f32,
) -> Option<(Entity, Vec3)> {
    nearest_match(registry, actors, center, radius, |entry, _, _| {
        is_just_of_class(entry.class_id, class_id)
    })
}

pub fn search_nearest_by_group<D: QueryFilter>(
    registry: &ActorRegistry,
    actors: &Query<(Entity, &Actor), D>,
    group: u8,
    center: Vec3,
    radius: f32,
) -> Option<(Entity, Vec3)> {
    nearest_match(registry, actors, center, radius, |entry, _, _| {
        entry.group == group
    })
}

pub fn delete_actor(world: &mut World, entity: Entity) -> bool {
    let proc_id = {
        let Some(mut registry) = world.get_resource_mut::<ActorRegistry>() else {
            return false;
        };
        let Some(entry) = registry.remove_entity(entity) else {
            return true;
        };
        if registry.is_creating(entry.proc_id) || entry.init_state == INIT_DELETING {
            return false;
        }
        registry.deleted_message();
        entry.proc_id
    };
    world.despawn(entity);
    let _ = proc_id;
    true
}

pub fn delete_actor_by_id(world: &mut World, proc_id: u32) -> bool {
    let entity = {
        let registry = world.get_resource::<ActorRegistry>();
        match registry.and_then(|registry| registry.entity(proc_id)) {
            Some(entity) => entity,
            None => return true,
        }
    };
    delete_actor(world, entity)
}

pub fn delete_actor_cmd(
    registry: &mut ActorRegistry,
    commands: &mut Commands,
    entity: Entity,
) -> bool {
    let Some(entry) = registry.remove_entity(entity) else {
        return true;
    };
    if registry.is_creating(entry.proc_id) || entry.init_state == INIT_DELETING {
        return false;
    }
    registry.deleted_message();
    let _ = commands.entity(entity).despawn();
    true
}

fn delete_pass(registry: &mut ActorRegistry, actors: &Query<(Entity, &Actor)>) {
    let stale: Vec<Entity> = registry
        .iter()
        .filter(|entry| actors.get(entry.entity).is_err())
        .map(|entry| entry.entity)
        .collect();
    for entity in stale {
        if let Some(entry) = registry.remove_entity(entity) {
            if entry.init_state == INIT_DELETING {
                registry.deleted_message();
            }
        }
    }
}

fn create_pass(registry: &mut ActorRegistry, actors: &Query<(Entity, &Actor)>) {
    let fresh: Vec<(Entity, String)> = actors
        .iter()
        .filter(|(entity, _)| !registry.is_registered(*entity))
        .map(|(entity, actor)| (entity, actor.name.clone()))
        .collect();
    for (entity, _) in fresh {
        if let Some(pending) = registry.pending.first().copied() {
            registry.finish_create(entity, pending.proc_id);
        } else {
            registry.register(entity, GROUP_ACTOR, -1);
        }
    }
}

fn priority_pass(registry: &mut ActorRegistry, actors: &Query<(Entity, &Actor)>) {
    let live: Vec<u32> = registry
        .iter()
        .filter(|entry| actors.get(entry.entity).is_ok())
        .map(|entry| entry.proc_id)
        .collect();
    let stale: Vec<u32> = registry
        .iter()
        .filter(|entry| actors.get(entry.entity).is_err())
        .map(|entry| entry.proc_id)
        .collect();
    for proc_id in stale {
        let _ = registry.remove(proc_id);
    }
    for proc_id in live {
        if let Some(entry) = registry.entry_mut(proc_id) {
            if entry.init_state == INIT_FIRST {
                entry.init_state = INIT_CREATING;
                entry.create_result = PHASE_LOADING;
            }
        }
    }
}

pub fn control(mut registry: ResMut<ActorRegistry>, actors: Query<(Entity, &Actor)>) {
    for pass in MANAGEMENT_PASSES {
        match pass {
            PASS_DELETE => delete_pass(&mut registry, &actors),
            PASS_PRIORITY => priority_pass(&mut registry, &actors),
            PASS_CREATE => create_pass(&mut registry, &actors),
            PASS_EXECUTE => {}
            PASS_DRAW => {}
            _ => {}
        }
    }
}

pub fn init(world: &mut World) {
    let mut registry = ActorRegistry::default();
    registry.init();
    world.insert_resource(registry);
}
