#![allow(dead_code, unused_imports)]
use once_cell::sync::Lazy;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, thread, time::Duration, any::Any};
use tokio::runtime::{Builder, Runtime};

use crate::async_task::{tm_async, AsyncTask};

pub trait AsAny {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
}
impl <T: 'static> AsAny for T {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[test]
fn main() {
    let runtime = tokio_runtime();

    let num = Rc::new(RefCell::new(0));
    let num_clone = num.clone();
    let _guard = runtime.enter();

    let task: AsyncTask<i32> = tm_async(async move {
        println!("[{:?}] Hello async task", thread::current().id());
        64
    })
    .then(|res| {
        println!("[{:?}] And then", thread::current().id());
        *num_clone.borrow_mut() = res;
    });

    let mut sync_task_queue: VecDeque<AsyncTask<i32>> = VecDeque::new();
    sync_task_queue.push_back(task);

    while let Some(task) = sync_task_queue.pop_front() {
        println!("[{:?}] main thread", thread::current().id());
        let mut some_task = task.process_if_finished(runtime);
        if let Some(st) = some_task.take() {
            sync_task_queue.push_back(st)
        } else {
            println!(
                "[{:?}] and then finish, get the num",
                thread::current().id()
            );
            assert_eq!(*num.borrow(), 64);
            break;
        }
    }
}

fn tokio_runtime() -> &'static Runtime {
    static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
        Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    });
    &TOKIO_RUNTIME
}
