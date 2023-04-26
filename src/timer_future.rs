#![allow(dead_code)]
use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    Future, FutureExt,
};
use std::{
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

////////////////////////// Future
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

pub struct SharedState {
    complete: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.complete {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            complete: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();

            println!("[{:?}] 计时任务完成.", thread::current().id());
            shared_state.complete = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

////////////////////////// Executor
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    fn run(&self) {
        println!("[{:?}] Executor starting running.", thread::current().id());

        while let Ok(task) = self.ready_queue.recv() {
            println!("[{:?}] Executor 接收到任务", thread::current().id());
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);

                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                    println!("[{:?}] Future was pending", thread::current().id());
                } else {
                    println!("[{:?}] Future was ready", thread::current().id());
                }
            }
        }

        println!("[{:?}] Executor stop running.", thread::current().id());
    }
}

pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        println!(
            "[{:?}] 将Future组成task, 放入channel中",
            thread::current().id()
        );
        self.task_sender.send(task).expect("Too many task queued");
    }
}

pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    task_sender: SyncSender<Arc<Task>>,
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!("[{:?}] wake_by_ref", thread::current().id());
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("Too many tasks queued")
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASK: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASK);
    println!("[{:?}] 生成Exectuor和Spawner", thread::current().id());
    (Executor { ready_queue }, Spawner { task_sender })
}

#[test]
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        TimerFuture::new(Duration::from_secs(3)).await;
    });

    drop(spawner);

    executor.run()
}
