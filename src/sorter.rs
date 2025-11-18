use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

type SortingCoro = Pin<Box<dyn Coroutine<(), Yield = List, Return = ()>>>;

pub const ALGORITHMS: &[Algorithm] = &[
    Algorithm::Insertion,
    Algorithm::Quick,
];

#[derive(Copy, Clone)]
pub enum Algorithm {
    // Bubble,
    // Selection,
    Insertion,
    Quick,
    //Heap,
    //Stalin,
    //Merge,
    //Tim,
}

impl Algorithm {
    pub fn func(&self) -> fn(Box<[usize]>) -> SortingCoro {
        match self {
            Algorithm::Insertion => |v| Box::pin(insertion(v)),
            Algorithm::Quick => |v| Box::pin(quicksort(v)),
        }
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Algorithm::Insertion => "insertion",
            Algorithm::Quick => "quicksort",
        })
    }
}

pub struct Sorter {
    func: SortingCoro,
    done: bool,
}

impl Sorter {
    pub fn new(s: SortingCoro) -> Self {
        Self {
            func: s,
            done: false
        }
    }

    pub fn iter(&mut self) -> Option<List> {
        if self.done {
            return None;
        }

        match self.func.as_mut().resume(()) {
            CoroutineState::Yielded(view) => Some(view),
            CoroutineState::Complete(_) => {
                self.done = true;
                None
            },
        }
    }
}

pub type List = Box<[usize]>;

// pub fn insertion(x: &mut List) -> Gen<List, (), _> {
//     r#gen!({
//         for i in 1..x.len() {
//             for j in 0..i {
//                 let j = i - j;
//                 if x[j - 1] <= x[j] {
//                     break;
//                 }
//                 x.swap(j - 1, j);
//                 yield_!(x.clone());
//             }
//         }
//     })
// }

// pub gen fn insertion(mut x: Vec<usize>) -> Box<[usize]> {
//     for i in 1..x.len() {
//         for j in 0..i {
//             let j = i - j;
//             if x[j - 1] <= x[j] {
//                 break;
//             }
//             x.swap(j - 1, j);
//             yield x.clone().into_boxed_slice();
//         }
//     }
// }

pub fn insertion(mut x: List) -> impl Coroutine<(), Yield = List, Return = ()> {
    #[coroutine] move || {
        for i in 1..x.len() {
            for j in 0..i {
                let j = i - j;
                if x[j - 1] <= x[j] {
                    break;
                }
                x.swap(j - 1, j);
                yield x.clone();
            }
        }
    }
}

pub fn quicksort(mut x: List) -> impl Coroutine<(), Yield = List, Return = ()> {
    #[coroutine] static move || {
        let l = x.len();
        let mut coro = Box::pin(_quicksort(&mut x, 0, l));
        while let CoroutineState::Yielded(y) = coro.as_mut().resume(()) {
            yield y;
        }
    }
}

pub fn _quicksort<'a>(x: &'a mut [usize], s: usize, e: usize) -> impl Coroutine<(), Yield = List, Return = ()> {
    #[coroutine] static move || {
        if x[s..e].len() <= 1 {
            return;
        }

        let pivot = s + qspartition(&mut x[s..e]);
        yield Box::from(&mut *x);

        let mut coro = Box::pin(_quicksort(x, s, pivot));
        while let CoroutineState::Yielded(y) = coro.as_mut().resume(()) {
            yield y;
        }

        // "cannot borrow x as mutable more than one time" well yes we can because
        // we're done with the coroutine
        std::mem::drop(coro);

        let mut coro = Box::pin(_quicksort(x, pivot + 1, e));
        while let CoroutineState::Yielded(y) = coro.as_mut().resume(()) {
            yield y;
        }
    }
}

// pub gen fn quicksort(mut x: Vec<usize>) -> Box<[usize]> {
//     for view in _quicksort(&mut x) {
//         yield view;
//     }
// }

// gen fn _quicksort(x: &mut [usize]) -> Box<[usize]> {
//     if x.len() <= 1 {
//         return;
//     }

//     let pivot = qspartition(x);
//     yield Box::from(&mut *x);

//     for view in Box::new(_quicksort(&mut x[..pivot])) {
//         yield view;
//     }

//     for view in Box::new(_quicksort(&mut x[pivot + 1..])) {
//         yield view;
//     }
// }


fn qspartition(x: &mut [usize]) -> usize {
    let pivot = x.len() / 2;
    x.swap(pivot, x.len() - 1);

    let mut i = 0;
    for j in 0..x.len() - 1 {
        if x[j] <= x[x.len() - 1] {
            x.swap(i, j);
            i += 1;
        }
    }

    x.swap(i, x.len() - 1);
    i
}
