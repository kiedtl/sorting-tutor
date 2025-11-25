pub mod heap;

use crate::for_coro;
use crate::utils::{Vew, Coro, VHeap};

use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

pub type List = Box<[usize]>;
pub type SortingCoro = Pin<Box<dyn Coroutine<(), Yield = Vew, Return = ()>>>;

pub const ALGORITHMS: &[Algorithm] = &[
    // First is default
    Algorithm::Heap,
    Algorithm::Bubble,

    Algorithm::Insertion,

    Algorithm::Selection,
    Algorithm::Quick,
];

#[derive(Copy, Clone)]
pub enum Algorithm {
    Bubble,
    Selection,
    Insertion,
    Quick,
    Heap,
    //Stalin,
    //Merge,
    //Tim,
}

impl Algorithm {
    pub fn func(&self) -> fn(Box<[usize]>) -> SortingCoro {
        match self {
            Algorithm::Bubble => |v| Box::pin(bubble(v)),
            Algorithm::Selection => |v| Box::pin(selection(v)),
            Algorithm::Insertion => |v| Box::pin(insertion(v)),
            Algorithm::Heap => |v| Box::pin(heap(v)),
            Algorithm::Quick => |v| Box::pin(quicksort(v)),
        }
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Algorithm::Bubble => "bubble",
            Algorithm::Selection => "selection",
            Algorithm::Insertion => "insertion",
            Algorithm::Heap => "heapsort",
            Algorithm::Quick => "quicksort",
        })
    }
}

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

pub fn bubble(mut x: List) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] move || {
        let mut n = x.len();

        loop {
            let mut work_done = true;

            for i in 1..n {
                if x[i - 1] > x[i] {
                    x.swap(i - 1, i);
                    work_done = true;
                    yield Vew::from(&x);
                }
            }

            if !work_done || n == 0 {
                break;
            }

            n -= 1;
        }
    }
}

pub fn selection(mut x: List) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] move || {
        for i in 0..(x.len() - 1) {
            let min = x
                .iter()
                .enumerate()
                .skip(i)
                .min_by_key(|&(_, &v)| v)
                .unwrap()
                .0;
            x.swap(i, min);
            yield Vew::from(&x);
        }
    }
}

pub fn insertion(mut x: List) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] move || {
        for i in 1..x.len() {
            for j in 0..i {
                let j = i - j;
                if x[j - 1] <= x[j] {
                    break;
                }
                x.swap(j - 1, j);
                yield Vew::from(&x);
            }
        }
    }
}

pub fn heap(mut x: List) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let mut h = heap::Heap::new(&mut x);

        for_coro!(view in build_heap(&mut h) =>
            yield view
        );

        for i in (1..h.nodes()).rev() {
            h.swap(heap::Node::of(0, &h), heap::Node::of(i, &h));
            yield VHeap::new(&h, Some((0, i))).into();

            h.abandon(1);
            for_coro!(y in heapify(h.root(), &mut h) => yield y);
        }
    }
}

fn build_heap(heap: &mut heap::Heap<'_>) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let k = heap.nodes() / 2;
        for i in (0..k).rev() {
            let n = heap::Node::of(i, heap);
            for_coro!(y in heapify(n, heap) => yield y);
        }
    }
}

// Based on Introduction to Algorithms, 6.2
fn heapify(
    node: heap::Node,
    heap: &mut heap::Heap<'_>
) -> impl Coroutine<(), Yield = Vew, Return = ()>
{
    #[coroutine] static move || {
        let value = node.value(heap);
        let children = node.children(heap);

        let max = [Some(node), children[0], children[1]]
            .into_iter()
            .filter_map(|n| n)
            .max_by_key(|n| n.value(heap))
            .unwrap();

        if max != node {
            heap.swap(node, max);
            yield VHeap::new(&heap, Some((node.index, max.index))).into();
            for_coro!(y in heapify(max, heap) => yield y);
        }
    }
}

pub fn quicksort(mut x: List) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let l = x.len();
        for_coro!(y in _quicksort(&mut x, 0, l) =>
            yield y
        );
    }
}

pub fn _quicksort<'a>(x: &'a mut [usize], s: usize, e: usize) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        if x[s..e].len() <= 1 {
            return;
        }

        let pivot = s + qspartition(&mut x[s..e]);
        yield Vew::from(&*x);

        for_coro!(y in _quicksort(x, s, pivot) => yield y);
        for_coro!(y in _quicksort(x, pivot + 1, e) => yield y);
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
