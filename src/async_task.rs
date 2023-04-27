#![allow(dead_code)]
use std::any::Any;

use futures::Future;
use tokio::{runtime::Runtime, task::JoinHandle};

pub struct AsyncTask<'a, T: Any> {
    join_handler: JoinHandle<T>,
    then: Option<Box<dyn FnOnce(T) + 'a>>,
}

impl<'a, T: Any> AsyncTask<'a, T> {
    pub fn process_if_finished(self, runtime: &Runtime) -> Option<Self> {
        if self.join_handler.is_finished() {
            let result = runtime.block_on(self.join_handler).unwrap();
            if let Some(then) = self.then {
                then(result);
            }
            None
        } else {
            Some(self)
        }
    }

    pub fn then<F>(mut self, then: F) -> Self 
    where F: FnOnce(T) + 'a {
        self.then = Some(Box::new(then));
        self
    }
}

pub fn tm_async<'a, T: Future + Send + Sync + 'static>(future: T) -> AsyncTask<'a, T::Output>
where
    T::Output: Send + Sync,
{
    let join_handler = tokio::spawn(future);
    AsyncTask { join_handler, then: None }
}
