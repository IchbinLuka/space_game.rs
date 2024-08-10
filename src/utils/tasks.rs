use std::{future::Future, pin::Pin};

use bevy::{
    ecs::system::Command,
    prelude::*,
    tasks::{block_on, futures_lite::future, IoTaskPool, Task},
};

// TODO: This should get easier when using the newer bevy version

#[derive(Component)]
pub struct TaskComponent<T>
where
    T: Send + 'static,
{
    pub task: Task<T>,
    pub on_complete: fn(T, &mut World),
}

pub fn poll_task<T>(mut commands: Commands, mut query: Query<(Entity, &mut TaskComponent<T>)>)
where
    T: Send + 'static,
{
    for (entity, mut task) in &mut query {
        let TaskComponent {
            ref mut task,
            on_complete,
        } = &mut *task;

        let on_complete = *on_complete;

        let Some(result) = block_on(future::poll_once(task)) else {
            continue;
        };

        commands.add(move |world: &mut World| on_complete(result, world));
        commands.entity(entity).despawn();
    }
}

pub struct StartJob<T>
where
    T: Send + 'static,
{
    pub job: Pin<Box<dyn Future<Output = T> + Send + 'static>>,
    pub on_complete: fn(T, &mut World),
}

impl<T> Command for StartJob<T>
where
    T: Send + 'static,
{
    fn apply(self, world: &mut World) {
        let task_pool = IoTaskPool::get();

        let task = task_pool.spawn(async move { async_compat::Compat::new(self.job).await });
        world.spawn(TaskComponent {
            task,
            on_complete: self.on_complete,
        });
    }
}
