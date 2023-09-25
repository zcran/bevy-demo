use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{animation::Animation, WINDOW_BOTTOM_Y, WINDOW_LEFT_X};

const PLAYER_VELOCITY_X: f32 = 400.0;
const PLAYER_VELOCITY_Y: f32 = 850.0;

const MAX_JUMP_HEIGHT: f32 = 230.0;
const MAX_JUMP_TIMES: i32 = 2;    // 最大跳跃次数

const SPRITESHEET_COLS: usize = 7;   // 7列
const SPRITESHEET_ROWS: usize = 8;   // 8行

const SPRITE_TILE_WIDTH: f32 = 128.0;   // 单位列宽
const SPRITE_TILE_HEIGHT: f32 = 256.0;  // 单位列高

const SPRITE_RENDER_WIDTH: f32 = 64.0;
const SPRITE_RENDER_HEIGHT: f32 = 128.0;    // 渲染时的缩放尺寸

const SPRITE_IDX_STAND: usize = 28; // 站立动作的精灵索引为28
const SPRITE_IDX_WALKING: &[usize] = &[7, 0];   // 行走动作的精灵索引为7和0两张
const SPRITE_IDX_JUMP: usize = 35;

const CYCLE_DELAY: Duration = Duration::from_millis(70);    // 70毫秒的动画延迟

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(movement)
            .add_system(jump)   // 包含二段跳的逻辑
            .add_system(rise)
            .add_system(fall)
            .add_system(apply_movement_animation)
            .add_system(apply_idle_sprite)
            .add_system(apply_jump_sprite)
            .add_system(update_direction)
            .add_system(update_sprite_direction)
            .add_system(key_counter_system);    // 二段跳的测试打印插件
    }
}

#[derive(Component)]
enum Direction {
    Right,
    Left,
}
#[derive(Component)]
struct KeyCounter {
    key: KeyCode,
    count: i32,
}

fn setup(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    server: Res<AssetServer>,
) {
    let image_handle: Handle<Image> = server.load("spritesheets/spritesheet_players.png");
    let texture_atlas = TextureAtlas::from_grid(
        image_handle,
        Vec2::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT),
        SPRITESHEET_COLS,
        SPRITESHEET_ROWS,
        None,
        None,
    );
    let atlas_handle = atlases.add(texture_atlas);

    commands
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(SPRITE_IDX_STAND),
            texture_atlas: atlas_handle,
            transform: Transform {
                translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 300.0, 0.0),
                scale: Vec3::new(
                    SPRITE_RENDER_WIDTH / SPRITE_TILE_WIDTH,
                    SPRITE_RENDER_HEIGHT / SPRITE_TILE_HEIGHT,
                    1.0,
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            SPRITE_TILE_WIDTH / 2.0,
            SPRITE_TILE_HEIGHT / 2.0,
        ))
        .insert(KinematicCharacterController::default())
        .insert(Direction::Right)
        .insert(KeyCounter {
            key: KeyCode::Up,
            count: 0,
        });
}

fn movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    let mut player = query.single_mut();

    let mut movement = 0.0;

    if input.pressed(KeyCode::Right) {
        movement += time.delta_seconds() * PLAYER_VELOCITY_X;
    }

    if input.pressed(KeyCode::Left) {
        movement += time.delta_seconds() * PLAYER_VELOCITY_X * -1.0;
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.0)),
    }
}

#[derive(Component)]
struct Jump(f32);

fn jump(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<
        (&mut KeyCounter, Entity, &KinematicCharacterControllerOutput),
        (With<KinematicCharacterController>, Without<Jump>),
    >,
) {
    if query.is_empty() {
        return;
    }

    for (mut counter, player, output) in query.iter_mut() {
        if input.pressed(KeyCode::Up) && counter.count < MAX_JUMP_TIMES {
            counter.count += 1;
            commands.entity(player).insert(Jump(0.0));  // 每次起跳都把jump设为0.0
        }

        if output.grounded {   // 落地才把jump计数器置0
            counter.count = 0;
        }
    }
}

fn rise(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&KeyCounter, Entity, &mut KinematicCharacterController, &mut Jump)>,
) {
    if query.is_empty() {
        return;
    }

    let (counter, entity, mut player, mut jump) = query.single_mut();

    let mut movement = time.delta().as_secs_f32() * PLAYER_VELOCITY_Y;   //上升的位移

    if movement + jump.0 >= MAX_JUMP_HEIGHT {   // 纠偏,并且在最大跳跃时禁止跳跃（移除跳跃）
        movement = MAX_JUMP_HEIGHT - jump.0;    // 超出距离为负，不足为补
        commands.entity(entity).remove::<Jump>(); // 一次跳跃完成，移除跳跃
    }

    if counter.count == (MAX_JUMP_TIMES - 1) {
        commands.entity(entity).remove::<Jump>(); // 达到多段跳跃上限，移除跳跃
    }

    jump.0 += movement; // 把位移赋值给jump，中间量，记录每次位移并叠加

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController, Without<Jump>>) {
    if query.is_empty() {
        return;
    }

    let mut player = query.single_mut();
    let movement = time.delta().as_secs_f32() * (PLAYER_VELOCITY_Y / 1.5) * -1.0;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn apply_movement_animation(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();
    if output.desired_translation.x != 0.0 && output.grounded {
        commands
            .entity(player)
            .insert(Animation::new(SPRITE_IDX_WALKING, CYCLE_DELAY));
    }
}

fn apply_idle_sprite(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &KinematicCharacterControllerOutput,
        &mut TextureAtlasSprite,
    )>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output, mut sprite) = query.single_mut();
    if output.desired_translation.x == 0.0 && output.grounded {
        commands.entity(player).remove::<Animation>();
        sprite.index = SPRITE_IDX_STAND
    }
}

fn apply_jump_sprite(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &KinematicCharacterControllerOutput,
        &mut TextureAtlasSprite
    )>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output, mut sprite) = query.single_mut();
    if !output.grounded {
        commands.entity(player).remove::<Animation>();
        sprite.index = SPRITE_IDX_JUMP
    }
}

fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

    if output.desired_translation.x > 0.0 {
        commands.entity(player).insert(Direction::Right);
    } else if output.desired_translation.x < 0.0 {
        commands.entity(player).insert(Direction::Left);
    }
}

fn update_sprite_direction(mut query: Query<(&mut TextureAtlasSprite, &Direction)>) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, direction) = query.single_mut();

    match direction {
        Direction::Right => sprite.flip_x = false,
        Direction::Left => sprite.flip_x = true,
    }
}

// 目前是测试用的系统，正式时可删去
fn key_counter_system(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&KeyCounter>,
) {
    if query.is_empty() {
        return;
    }

    let counter = query.single();

    if keyboard_input.just_pressed(KeyCode::Up) { 
        println!("Key {:?} pressed {} times", counter.key, counter.count);
    }
}
