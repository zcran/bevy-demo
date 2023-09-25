// bevy将 时间、声音、纹理、网格、渲染器 作为全局ECS资源

use bevy::prelude::*;

#[derive(Component)]    // 人 组件
struct Person;

#[derive(Component)]        // 名字 组件
struct Name(String);

// 添加人和人名
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("CC".to_string())));
    commands.spawn((Person, Name("LILY".to_string())));
    commands.spawn((Person, Name("LULU".to_string())));
}


#[derive(Resource)]
struct GreetTimer(Timer);   // 跟踪流逝的时间

// 问候人名  
fn greet_people(
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    query: Query<&Name, With<Person>>) {       // Res和ResMut指针（分别）提供对资源的读写访问权限。
    if timer.0.tick(time.delta()).just_finished() {     // delta字段提供上次更新以来已经过去的时间
        for name in query.iter() {
            println!("hello {}!", name.0);
    }
    }
}

// 你好 组件
pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))     // 使用TimerMode::Repeating使计时器重复
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}





fn main() {
    App::new()
    .add_plugins(DefaultPlugins)    // 默认插件系统, 具有事件循环，ECS时间在每个帧循环运作一次
    .add_plugin(HelloPlugin)    // 你好 自制组件
    .run();
}
