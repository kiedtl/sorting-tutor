use leptos::prelude::*;

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

// A "snapshot" of progress of a sorting algorithm, at the very least containing
// the partially sorted list and possibly also annotations, tree structures, etc.
// Not to be confused with Leptos' View/IntoView stuff.
pub struct Vew {
    inner: Box<dyn IsVew>,
}

impl Vew {
    pub fn list(&self) -> &[usize] {
        self.inner.list()
    }

    pub fn into_view(&self) -> AnyView {
        self.inner.into_view()
    }
}

impl From<&[usize]> for Vew {
    fn from(f: &[usize]) -> Vew {
        // Incredibly wasteful. Vew should just be an enum
        Vew { inner: Box::new(VList(Box::from(f))) }
    }
}

impl From<&Box<[usize]>> for Vew {
    fn from(f: &Box<[usize]>) -> Vew {
        Vew { inner: Box::new(VList(f.clone())) }
    }
}

pub trait IsVew: Send + Sync {
    fn list(&self) -> &[usize];
    fn into_view(&self) -> AnyView;
}

pub struct VList(Box<[usize]>);

impl IsVew for VList {
    fn list(&self) -> &[usize] {
        &self.0
    }

    fn into_view(&self) -> AnyView {
        let s = self.0.clone();
        view! {
            <table>
                <tr>
                {move || s.iter().copied().map(|v| view! {
                    <td>{v}</td>
                }).collect_view()}
                </tr>
            </table>
        }.into_any()
    }
}
