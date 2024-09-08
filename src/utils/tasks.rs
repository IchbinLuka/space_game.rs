use std::future::Future;

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::IoTaskPool};
use cfg_if::cfg_if;

// TODO: This should get easier when using the newer bevy version

#[derive(Component)]
pub struct TaskComponent {
    receiver: crossbeam_channel::Receiver<CommandQueue>,
}

impl TaskComponent {
    pub fn new<T>(
        #[cfg(not(target_family = "wasm"))] future: impl Future<Output = T> + 'static + Send,
        #[cfg(target_family = "wasm")] future: impl Future<Output = T> + 'static,
        on_complete: impl FnOnce(T, &mut World) + Send + 'static,
    ) -> TaskComponent
    where
        T: Send + 'static,
    {
        let task_pool = IoTaskPool::get();
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = task_pool.spawn(async move {
            cfg_if! {
                if #[cfg(not(target_family = "wasm"))] {
                    let future = async_compat::Compat::new(future);
                }
            }

            let result = future.await;
            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| (on_complete)(result, world));
            tx.send(command_queue).unwrap();
        });
        Self { receiver: rx }
    }
}

fn poll_task(mut commands: Commands, query: Query<(Entity, &TaskComponent)>) {
    for (entity, task) in &query {
        let Ok(ref mut queue) = task.receiver.try_recv() else {
            continue;
        };

        commands.append(queue);
        commands.entity(entity).despawn();
    }
}

pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, poll_task);
    }
}
