use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

#[macro_export]
macro_rules! for_coro {
    ($value:ident in $coro:expr => $b:expr) => {
        {
            let mut coro = Box::pin($coro);
            while let CoroutineState::Yielded($value) = coro.as_mut().resume(()) {
                $b
            }
        }
    };
}

pub struct Coro<Y> {
    func: Pin<Box<dyn Coroutine<(), Yield = Y, Return = ()>>>,
    done: bool,
}

impl<Y> Coro<Y> {
    // pub fn of<S>(s: S) -> Self
    // where
    //     S: Coroutine<(), Yield = Y, Return = ()>,
    // {
    //     Self {
    //         func: Box::pin(s),
    //         done: false
    //     }
    // }

    pub fn new(s: Pin<Box<dyn Coroutine<(), Yield = Y, Return = ()>>>) -> Self {
        Self {
            func: s,
            done: false
        }
    }
}

impl<Y> Iterator for Coro<Y> {
    type Item = Y;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.func.as_mut().resume(()) {
            CoroutineState::Yielded(y) => Some(y),
            CoroutineState::Complete(_) => {
                self.done = true;
                None
            },
        }
    }
}
