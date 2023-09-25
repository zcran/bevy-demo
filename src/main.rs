// bevy将 时间、声音、纹理、网格、渲染器 作为全局ECS资源
use bevy::prelude::*;
fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup)
    .add_system(sprite_movement)
    .run();
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
    Left, 
    Right,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());  // 2d投影
    commands.spawn((    // 直接录入元组
    SpriteBundle {
      texture: asset_server.load("branding/ferris.png"),   // 图形
      transform: Transform::from_xyz(100., 0., 0.),     // 坐标
      ..default()
    },
    Direction::Up,    // 初始移动方向
    ));     
}

fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
            Direction::Left => transform.translation.x -= 150. * time.delta_seconds(),
            Direction::Right => transform.translation.x += 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Left;
        } else if transform.translation.x < -200. {
            *logo = Direction::Right;
        } else if transform.translation.x > 200. {
            *logo = Direction::Up;
        }
    }
}




