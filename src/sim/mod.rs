pub mod actor;
pub mod ajav;
pub mod area;
pub mod arrow;
pub mod balloon;
pub mod barrier;
pub mod barrel;
pub mod barricade;
pub mod bird;
pub mod buoybarrel;
pub mod bomb;
pub mod bombflower;
pub mod boomerang;
pub mod chest;
pub mod carry;
pub mod cleared;
pub mod clock;
pub mod collision;
pub mod combat;
pub mod damage;
pub mod death;
pub mod door;
pub mod ending;
pub mod enemy;
pub mod event;
pub mod events;
pub mod fallrock;
pub mod fire;
pub mod flags;
pub mod frame;
pub mod ghostship;
pub mod ground;
pub mod gull;
pub mod hammer;
pub mod hammerhit;
pub mod health;
pub mod homen;
pub mod hookshot;
pub mod hop;
pub mod ice;
pub mod intent;
pub mod inventory;
pub mod item;
pub mod jump;
pub mod ladder;
pub mod koisi;
pub mod layer;
pub mod leaves;
pub mod lock;
pub mod marlon;
pub mod mflft;
pub mod msg;
pub mod pickup;
pub mod player;
pub mod raft;
pub mod rockfall;
pub mod roll;
pub mod scenery;
pub mod schedule;
pub mod select;
pub mod shop;
pub mod sea;
pub mod ship;
pub mod shmrgrd;
pub mod statue;
pub mod stone;
pub mod stream;
pub mod swim;
pub mod swhammer;
pub mod tag;
pub mod trapdoor;
pub mod talk;
pub mod tingle;
pub mod torch;
pub mod transition;
pub mod tsubo;
pub mod volcano;
pub mod walk;
pub mod wall;
pub mod wind;
pub mod windmill;
pub mod warship;
pub mod water;
pub mod zelda;

use bevy_ecs::prelude::*;
use glam::Vec3;
use repame_sim::Sim;

/// Per-tick pipeline order: input read, process execute, frame counters.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WwSimSet {
    Input,
    Execute,
    After,
}

pub(crate) fn spawn_actors(world: &mut World, tables: &crate::dzs::StageTables) {
    for source in tables.actors() {
        spawn_actor(world, source);
    }
}

pub(crate) fn spawn_actor(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = actor::Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
    };
    if source.name == ship::SHIP_ACTOR {
        world.spawn((actor, ship::Ship::default())).id()
    } else if source.name == enemy::ENEMY_ACTOR {
        let (state, hit_points) = enemy::spawn(source.parameters, source.angle[1]);
        world
            .spawn((actor, state, hit_points, enemy::Hurt::default()))
            .id()
    } else if source.name == enemy::LANCE_ACTOR {
        if let Some((state, hit_points)) = enemy::spawn_lance(source.parameters, source.angle[1]) {
            world
                .spawn((actor, state, hit_points, enemy::Hurt::default()))
                .id()
        } else {
            world.spawn(actor).id()
        }
    } else if source.name == warship::SHIP_ACTOR
        && warship::is_switched(source.parameters, source.angle[0], world.resource())
    {
        world.spawn((actor, warship::spawn(source.angle[1]))).id()
    } else if source.name == fire::FIRE_ACTOR {
        world
            .spawn((actor, fire::spawn(source.parameters, world.resource())))
            .id()
    } else if source.name == gull::GULL_ACTOR {
        let state = gull::spawn(source.parameters, &actor);
        world.spawn((actor, state)).id()
    } else if source.name == cleared::CLEARED_ACTOR {
        world
            .spawn((actor, cleared::spawn(source.parameters, world.resource())))
            .id()
    } else if source.name == balloon::BALLOON_ACTOR {
        let (state, hit_points) = balloon::spawn(source.parameters);
        world
            .spawn((actor, state, hit_points, enemy::Hurt::default()))
            .id()
    } else if pickup::is_pickup(&source.name) {
        world
            .spawn((actor, pickup::Pickup::spawn(source.parameters)))
            .id()
    } else if source.name == talk::TALK_ACTOR {
        let speaker = talk::Talk::new(talk::message(source.angle[0]), source.room);
        world.spawn((actor, speaker)).id()
    } else if flags::is_event_switch(&source.name) {
        let switch = flags::EventSwitch::spawn(source.parameters);
        world.spawn((actor, switch)).id()
    } else if scenery::is_scenery(&source.name) {
        world.spawn((actor, scenery::Scenery)).id()
    } else if tsubo::is_tsubo(&source.name) {
        let pot = tsubo::Tsubo::spawn(source.parameters);
        world.spawn((actor, pot, carry::Carryable::Tsubo)).id()
    } else if tag::is_tag(&source.name) {
        let marker = tag::Tag::spawn(source.parameters, actor.pos, source.angle[1]);
        world.spawn((actor, marker)).id()
    } else if source.name == carry::PIG_ACTOR {
        world.spawn((actor, carry::Carryable::Pig)).id()
    } else if source.name == bird::BIRD_ACTOR {
        let (state, hit_points) = bird::spawn();
        world.spawn((actor, state, hit_points)).id()
    } else if source.name == marlon::MARLON_ACTOR {
        let friend = marlon::Marlon::spawn(source.parameters);
        world.spawn((actor, friend)).id()
    } else if source.name == shop::SHOP_ACTOR {
        let store = shop::Shop::spawn(source.parameters);
        world.spawn((actor, store)).id()
    } else if source.name == zelda::ZELDA_ACTOR {
        let princess = zelda::Zelda::spawn(source.parameters);
        world.spawn((actor, princess)).id()
    } else if source.name == tingle::TINGLE_ACTOR {
        world.spawn((actor, tingle::Tingle::spawn())).id()
    } else if chest::is_chest(&source.name) {
        let box_ = chest::Chest::spawn(source.parameters, source.angle[2], source.room);
        world.spawn((actor, box_)).id()
    } else if stone::is_stone(&source.name) {
        let rock = stone::Stone::spawn(source.parameters);
        world.spawn((actor, rock, carry::Carryable::Stone)).id()
    } else if source.name == wall::WALL_ACTOR {
        let barrier = wall::Wall::spawn(source.parameters);
        world.spawn((actor, barrier)).id()
    } else if bomb::is_bomb(&source.name) {
        let shell = bomb::Bomb::spawn(source.parameters);
        world.spawn((actor, shell)).id()
    } else if ladder::is_ladder(&source.name) {
        let climb = ladder::Ladder::spawn(source.parameters);
        world.spawn((actor, climb)).id()
    } else if arrow::is_arrow(&source.name) {
        let mut shaft = arrow::Arrow::nocked(arrow::TYPE_NORMAL, 0);
        shaft.pos = actor.pos;
        shaft.parameter = source.parameters;
        world.spawn((actor, shaft)).id()
    } else if koisi::is_koisi(&source.name) {
        let boulder = koisi::Koisi::spawn(source.parameters);
        world.spawn((actor, boulder, carry::Carryable::Stone)).id()
    } else if door::Door::is_shutter(&source.name) {
        let hatch = door::Door::spawn(source.parameters, source.angle, door::Kind::Shutter);
        world.spawn((actor, hatch)).id()
    } else if door::Door::is_sliding(&source.name) {
        let gate = door::Door::spawn(source.parameters, source.angle, door::Kind::Slide);
        world.spawn((actor, gate)).id()
    } else if door::Door::is_key_door(&source.name) {
        let gate = door::Door::spawn(source.parameters, source.angle, door::Kind::Key);
        world.spawn((actor, gate)).id()
    } else if source.name == ghostship::GHOSTSHIP_ACTOR {
        let wreck = ghostship::Ghostship::spawn(source.parameters);
        world.spawn((actor, wreck)).id()
    } else if bombflower::is_flower(&source.name) {
        let bloom = bombflower::Bombflower::spawn(
            source.parameters,
            world.resource::<flags::Switches>().test(((source.parameters >> 8) & 0xFF) as u8),
        );
        world.spawn((actor, bloom)).id()
    } else if source.name == bombflower::BOMB_ACTOR {
        let crawling = bombflower::Crawler::spawn(source.parameters);
        world.spawn((actor, crawling)).id()
    } else if barrel::is_barrel(&source.name) {
        let cask = barrel::Barrel::spawn(source.parameters);
        world.spawn((actor, cask, carry::Carryable::Barrel)).id()
    } else if rockfall::is_rockfall(&source.name) {
        let shower = rockfall::Rockfall::spawn(source.parameters);
        world.spawn((actor, shower)).id()
    } else if fallrock::is_fallrock(&source.name) {
        let stone = fallrock::FallRock::spawn(1.0);
        world.spawn((actor, stone)).id()
    } else if wind::is_wind_tag(&source.name) {
        let gale = wind::WindTag::spawn(source.parameters, source.angle[2]);
        world.spawn((actor, gale)).id()
    } else if windmill::is_windmill(&source.name) {
        let tower = windmill::Windmill::spawn(source.parameters);
        world.spawn((actor, tower)).id()
    } else if buoybarrel::is_buoy_barrel(&source.name) {
        let float = buoybarrel::BuoyBarrel::spawn(source.parameters, actor.pos);
        world.spawn((actor, float)).id()
    } else if ice::is_ice(&source.name) {
        let block = ice::Ice::spawn(source.parameters, world.resource::<flags::Switches>());
        world.spawn((actor, block)).id()
    } else if ajav::is_ajav(&source.name) {
        match ajav::Ajav::spawn(
            source.parameters,
            world.resource::<flags::StoryBits>(),
            world.resource::<flags::Switches>(),
        ) {
            Some(wall) => world.spawn((actor, wall)).id(),
            None => world.spawn(actor).id(),
        }
    } else if homen::is_domen(&source.name) {
        match homen::Domen::spawn(
            source.parameters,
            world.resource::<flags::Switches>(),
            source.angle,
        ) {
            Some(mask) => world.spawn((actor, mask)).id(),
            None => world.spawn(actor).id(),
        }
    } else if leaves::is_leaves(&source.name) {
        let pile = leaves::Leaves::spawn(source.parameters, world.resource::<flags::Switches>());
        world.spawn((actor, pile)).id()
    } else if volcano::is_volcano(&source.name) {
        let crater = volcano::Volcano::spawn(
            source.parameters,
            actor.pos,
            world.resource::<flags::Switches>(),
        );
        world.spawn((actor, crater)).id()
    } else if swhammer::is_swhammer(&source.name) {
        let set = world
            .resource::<flags::Switches>()
            .test(((source.parameters >> 8) & 0xFF) as u8);
        let plunger = swhammer::Swhammer::spawn(source.parameters, set);
        world.spawn((actor, plunger)).id()
    } else if shmrgrd::is_guard(&source.name) {
        let set = world
            .resource::<flags::Switches>()
            .test(((source.parameters >> 8) & 0xFF) as u8);
        let guard = shmrgrd::Shmrgrd::spawn(source.parameters, set);
        world.spawn((actor, guard)).id()
    } else if statue::is_statue(&source.name) {
        let idol = statue::Statue::spawn(source.parameters);
        world.spawn((actor, idol)).id()
    } else if swpush::is_swpush(&source.name) {
        let button = swpush::Swpush::spawn(
            source.parameters,
            source.angle[2],
            world.resource::<flags::Switches>(),
        );
        world.spawn((actor, button)).id()
    } else if mflft::is_mflft(&source.name) {
        let hoist = mflft::Mflft::spawn(
            source.parameters,
            actor.pos,
            world.resource::<flags::Switches>(),
            world.get_resource::<collision::Collision>(),
        );
        world.spawn((actor, hoist)).id()
    } else if taglight::is_tag_light(&source.name) {
        let spot = taglight::TagLight::spawn(source.parameters);
        world.spawn((actor, spot)).id()
    } else if torch::is_torch(&source.name) {
        let brand = torch::Torch::spawn(source.parameters, 1.0);
        world.spawn((actor, brand)).id()
    } else if trapdoor::is_trapdoor(&source.name) {
        let hatch = trapdoor::Trapdoor::spawn();
        world.spawn((actor, hatch)).id()
    } else if barricade::is_barricade(&source.name) {
        let fence = barricade::Barricade::spawn(source.parameters, world.resource::<flags::Switches>());
        world.spawn((actor, fence)).id()
    } else if raft::is_raft(&source.name) {
        let vessel = raft::Raft::spawn(source.parameters, source.angle);
        world.spawn((actor, vessel)).id()
    } else if barrier::is_barrier(&source.name) {
        let field = barrier::Barrier::spawn(Vec3::ONE);
        world.spawn((actor, field)).id()
    } else if hookshot::is_hookshot(&source.name) {
        let chain = hookshot::Hookshot::nocked(actor.pos);
        world.spawn((actor, chain)).id()
    } else if source.name == door::JAIL_ACTOR {
        let cell = door::JailDoor::spawn(source.parameters, world.resource::<flags::Switches>().test((source.parameters & 0xFF) as u8));
        world.spawn((actor, cell)).id()
    } else {
        world.spawn(actor).id()
    }
}

pub(crate) fn clear_actors(world: &mut World) {
    let mut query = world.query::<(Entity, &actor::Actor)>();
    let stale: Vec<Entity> = query.iter(world).map(|(entity, _)| entity).collect();
    for entity in stale {
        let _ = world.despawn(entity);
    }
    clear_shots(world);
}

pub(crate) fn clear_shots(world: &mut World) {
    let mut query = world.query::<(Entity, &item::InFlight)>();
    let flying: Vec<Entity> = query.iter(world).map(|(entity, _)| entity).collect();
    for entity in flying {
        let _ = world.despawn(entity);
    }
}

pub fn build(
    sim: &mut Sim,
    stage: Option<collision::StageData>,
    tables: Option<&crate::dzs::StageTables>,
) {
    sim.world.insert_resource(frame::CurrentFrame(0));
    sim.world.insert_resource(intent::MoveIntent::default());
    sim.world.insert_resource(intent::CameraYaw::default());
    sim.world.insert_resource(intent::DamageIntent::default());
    sim.world.insert_resource(intent::DoIntent::default());
    sim.world.insert_resource(intent::TargetIntent::default());
    sim.world.insert_resource(intent::ItemIntent::default());
    sim.world.insert_resource(intent::UseIntent::default());
    sim.world.insert_resource(intent::GameOverChoice::default());
    sim.world.insert_resource(health::Health::default());
    sim.world.insert_resource(windmill::Wind::default());
    sim.world.insert_resource(events::SimEvents::default());
    sim.world.insert_resource(events::Stride::default());
    sim.world.insert_resource(water::Water::default());
    sim.world.insert_resource(swim::Swim::default());
    sim.world.insert_resource(roll::Roll::default());
    sim.world.insert_resource(hop::Hop::default());
    sim.world.insert_resource(lock::Lock::default());
    sim.world.insert_resource(jump::Jump::default());
    sim.world.insert_resource(combat::Combat::default());
    sim.world.insert_resource(hammer::Cut::default());
    sim.world.insert_resource(pickup::Rupees::default());
    sim.world.insert_resource(item::Held::default());
    sim.world.insert_resource(talk::Dialogue::default());
    sim.world.insert_resource(flags::EventFlags::default());
    sim.world.insert_resource(flags::OrderFlags::default());
    sim.world.insert_resource(flags::ZoneActors::default());
    sim.world.insert_resource(inventory::Equipped::default());
    sim.world.insert_resource(inventory::Got::default());
    sim.world.insert_resource(clock::Clock::default());
    sim.world.insert_resource(schedule::Schedule::from_header(
        tables.and_then(|tables| tables.stag()).as_ref(),
    ));
    sim.world.insert_resource(select::Select::default());
    sim.world.insert_resource(carry::Carry::default());
    sim.world.insert_resource(chest::Chests::default());
    sim.world.insert_resource(flags::StoryBits::default());
    sim.world.insert_resource(flags::Switches::default());
    sim.world.insert_resource(flags::EventReg::default());
    sim.world.insert_resource(flags::Charts::default());
    sim.world.insert_resource(flags::Arrivals::default());
    sim.world.insert_resource(ending::Ending::default());
    let event_list = tables
        .and_then(|tables| crate::assets::load_event_list(tables.name()))
        .unwrap_or_default();
    sim.world.insert_resource(event_list);
    sim.world.insert_resource(event::ActiveEvent::default());
    sim.world.insert_resource(event::ItemPartner::default());
    sim.world.insert_resource(crate::assets::load_link_anms().unwrap_or_default());
    sim.world.insert_resource(msg::MsgBox::default());
    sim.world.insert_resource(crate::assets::load_msg_table().unwrap_or_default());
    sim.world.insert_resource(death::GameOver::default());
    sim.world.insert_resource(transition::NextStage::default());
    let mut area = area::Area::default();
    if let Some(stage) = stage {
        area.spawn = stage.spawn;
        let (bounds_min, bounds_max) = stage.collision.bounds();
        area.bounds_min = bounds_min;
        area.bounds_max = bounds_max;
        area.yaw = stage.yaw;
        sim.world.insert_resource(stage.collision);
    }
    if let Some(tables) = tables.filter(|tables| tables.has_sea_actor()) {
        let sea_tables = sea::SeaTables::new(|room| tables.wave_max(room));
        sim.world.insert_resource(sea::Sea::new(sea_tables));
    }
    if let Some(tables) = tables {
        spawn_actors(&mut sim.world, tables);
    }
    let spawn = area.spawn;
    let yaw = area.yaw;
    sim.world.insert_resource(area);
    sim.world.insert_resource(area::StageName(
        tables
            .map(|tables| tables.name().to_owned())
            .unwrap_or_default(),
    ));
    sim.world.insert_resource(collision::MoveFrom(spawn));
    sim.world.spawn((
        player::Player {
            pos: spawn,
            yaw,
            face: yaw,
            speed: 0.0,
            speed_y: 0.0,
            drift: Vec3::ZERO,
        },
        hammer::Hammer::idle(),
    ));
    sim.add_chained_systems(
        (
            (
                (ajav::control.in_set(WwSimSet::Execute),).chain(),
                (
                    death::freeze.in_set(WwSimSet::Execute),
                    carry::control.in_set(WwSimSet::Execute),
                    tsubo::control.in_set(WwSimSet::Execute),
                    koisi::control.in_set(WwSimSet::Execute),
                    bomb::control.in_set(WwSimSet::Execute),
                    bombflower::flower_control.in_set(WwSimSet::Execute),
                    bombflower::crawler_control.in_set(WwSimSet::Execute),
                    ladder::control.in_set(WwSimSet::Execute),
                    arrow::control.in_set(WwSimSet::Execute),
                    hookshot::control.in_set(WwSimSet::Execute),
                    barrier::control.in_set(WwSimSet::Execute),
                    raft::control.in_set(WwSimSet::Execute),
                    barricade::control.in_set(WwSimSet::Execute),
                    trapdoor::control.in_set(WwSimSet::Execute),
                    torch::control.in_set(WwSimSet::Execute),
                    barrel::control.in_set(WwSimSet::Execute),
                    fallrock::control.in_set(WwSimSet::Execute),
                    windmill::control.in_set(WwSimSet::Execute),
                    buoybarrel::control.in_set(WwSimSet::Execute),
                    ice::control.in_set(WwSimSet::Execute),
                )
                    .chain(),
                (
                    door::control.in_set(WwSimSet::Execute),
                    door::jail_control.in_set(WwSimSet::Execute),
                    bird::control.in_set(WwSimSet::Execute),
                    tingle::control.in_set(WwSimSet::Execute),
                    select::control.in_set(WwSimSet::Execute),
                )
                    .chain(),
                (
                    talk::control.in_set(WwSimSet::Execute),
                    flags::control.in_set(WwSimSet::Execute),
                    clock::control.in_set(WwSimSet::Execute),
                    ghostship::control.in_set(WwSimSet::Execute),
                    volcano::control.in_set(WwSimSet::Execute),
                    event::control.in_set(WwSimSet::Execute),
                    msg::control.in_set(WwSimSet::Execute),
                    ending::control.in_set(WwSimSet::Execute),
                    ground::ground_check.in_set(WwSimSet::Execute),
                    water::water_probe.in_set(WwSimSet::Execute),
                    swim::control.in_set(WwSimSet::Execute),
                    lock::control.in_set(WwSimSet::Execute),
                    jump::control.in_set(WwSimSet::Execute),
                    roll::control.in_set(WwSimSet::Execute),
                    hop::control.in_set(WwSimSet::Execute),
                )
                    .chain(),
                (
                    combat::control.in_set(WwSimSet::Execute),
                    item::control.in_set(WwSimSet::Execute),
                    boomerang::control.in_set(WwSimSet::Execute),
                    enemy::control.in_set(WwSimSet::Execute),
                    balloon::control.in_set(WwSimSet::Execute),
                    fire::control.in_set(WwSimSet::Execute),
                    gull::control.in_set(WwSimSet::Execute),
                    hammer::control.in_set(WwSimSet::Execute),
                    hammerhit::control.in_set(WwSimSet::Execute),
                )
                    .chain(),
                (
                    swhammer::control.in_set(WwSimSet::Execute),
                    shmrgrd::control.in_set(WwSimSet::Execute),
                    statue::control.in_set(WwSimSet::Execute),
                    schedule::control.in_set(WwSimSet::Execute),
                    taglight::control.in_set(WwSimSet::Execute),
                    rockfall::control.in_set(WwSimSet::Execute),
                    wind::control.in_set(WwSimSet::Execute),
                    homen::control.in_set(WwSimSet::Execute),
                    leaves::control.in_set(WwSimSet::Execute),
                    swpush::control.in_set(WwSimSet::Execute),
                    mflft::control.in_set(WwSimSet::Execute),
                )
                    .chain(),
            )
                .chain(),
            (
                pickup::control.in_set(WwSimSet::Execute),
                collision::remember_move_from.in_set(WwSimSet::Execute),
                walk::walk.in_set(WwSimSet::Execute),
                swim::surface.in_set(WwSimSet::Execute),
                ship::control.in_set(WwSimSet::Execute),
                area::clamp_to_area.in_set(WwSimSet::Execute),
                transition::check.in_set(WwSimSet::Execute),
                sea::sea_tick.in_set(WwSimSet::Execute),
                warship::control.in_set(WwSimSet::Execute),
                health::apply_damage.in_set(WwSimSet::Execute),
                death::check.in_set(WwSimSet::Execute),
                cleared::control.in_set(WwSimSet::Execute),
                frame::advance.in_set(WwSimSet::After),
            )
                .chain(),
        )
            .chain(),
    );
}
