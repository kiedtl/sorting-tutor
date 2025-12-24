pub mod heap;
pub mod merge;
pub mod quick;
pub mod shell;

use crate::for_coro;
use crate::recorder::Recorder;
use crate::utils::{ Coro, IsSnapshot, Snapshot, VList, VHeap };

use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

pub type List = Box<[usize]>;
pub type SortingCoro = Pin<Box<dyn Coroutine<(), Yield = Snapshot, Return = ()>>>;

pub const ALGORITHMS: &[Algorithm] = &[
    Algorithm::Selection,
    Algorithm::Bubble,
    Algorithm::Insertion,
    Algorithm::Shell,
    Algorithm::Quick,
    Algorithm::Heap,
    Algorithm::Merge,
];

#[derive(Copy, Clone, Default)]
pub enum Algorithm {
    Bubble,
    Selection,
    Insertion,
    Shell,
    Quick,
    #[default]
    Heap,
    //Stalin,
    Merge,
    //Tim,
    //Drift,
}

impl Algorithm {
    pub fn func(&self) -> fn(Box<[usize]>, Recorder) -> SortingCoro {
        match self {
            Algorithm::Bubble => |v, r| Box::pin(bubble(v, r)),
            Algorithm::Selection => |v, r| Box::pin(selection(v, r)),
            Algorithm::Insertion => |v, r| Box::pin(insertion(v, r)),
            Algorithm::Shell => |v, r| Box::pin(shell::sort(v, r)),
            Algorithm::Heap => |v, r| Box::pin(heap(v, r)),
            Algorithm::Quick => |v, r| Box::pin(quick::sort(v, r)),
            Algorithm::Merge => |v, r| Box::pin(merge::sort(v, r)),
        }
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Algorithm::Bubble => "bubble",
            Algorithm::Selection => "selection",
            Algorithm::Insertion => "insertion",
            Algorithm::Shell => "shellsort",
            Algorithm::Heap => "heapsort",
            Algorithm::Quick => "quicksort",
            Algorithm::Merge => "mergesort",
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

pub fn bubble(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("bubble");
        yield VList::new(&x).into();
        let mut n = x.len();

        loop {
            let mut work_done = true;

            for i in 1..n {
                if r.gt(x[i - 1], x[i]) {
                    r.swap(&mut x, i - 1, i);
                    work_done = true;
                    yield VList::new(&x)
                        .swapped(i - 1, i)
                        .into();
                }
            }

            if !work_done || n == 0 {
                break;
            }

            n -= 1;
        }
    }
}

pub fn selection(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("selection");
        yield VList::new(&x).into();

        for i in 0..(x.len() - 1) {
            // let min = (i..x.len())
            //     .reduce(|a, v| r.min(a, v, |v| x[v]))
            //     .unwrap();

            let mut min = i;
            for j in (i + 1)..x.len() {
                if r.lt(x[j], x[min]) {
                    min = j;
                }

                yield VList::new(&x)
                    .current(j)
                    .special(min)
                    .into_snapshot()
                    .fleeting();
            }

            r.swap(&mut x, i, min);
            yield VList::new(&x)
                .swapped(i, min)
                .special(min)
                .into();
        }
    }
}

pub fn insertion(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("insertion");
        yield VList::new(&x).into();

        for i in 1..x.len() {
            for j in 0..i {
                let j = i - j;
                if !r.gt(x[j - 1], x[j]) {
                    break;
                }
                r.swap(&mut x, j - 1, j);
                yield VList::new(&x)
                    .swapped(j - 1, j)
                    .into();
            }
        }
    }
}

pub fn heap(mut x: List, r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("heapsort");
        let mut h = heap::Heap::new(&mut x);
        yield VHeap::new(&h, None).into();

        for_coro!(view in build_heap(&mut h, r) =>
            yield view
        );

        for i in (1..h.nodes()).rev() {
            h.swap(heap::Node::of(0, &h), heap::Node::of(i, &h), r);
            yield VHeap::new(&h, Some((0, i)))
                .now_heapifying(0)
                .expl("Moving root to end of heap")
                .expl(format!("Next: sift heap at {}", h.root().value(&h)))
                .into();

            h.abandon(1);
            for_coro!(y in heapify(h.root(), &mut h, r) =>
                yield y.expl("Sifting heap").into()
            );
        }
    }
}

fn build_heap(heap: &mut heap::Heap<'_>, r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("build_heap");
        let k = heap.nodes() / 2;
        for i in (0..k).rev() {
            let n = heap::Node::of(i, heap);
            for_coro!(y in heapify(n, heap, r) =>
                yield y.expl("Building heap").into()
            );
        }
    }
}

// Based on Introduction to Algorithms, 6.2
fn heapify(
    node: heap::Node,
    heap: &mut heap::Heap<'_>,
    mut r: Recorder,
) -> impl Coroutine<(), Yield = VHeap, Return = ()>
{
    #[coroutine] static move || {
        let _g = r.f("heapify");
        let value = node.value(heap);
        let children = node.children(heap);

        let max = [Some(node), children[0], children[1]]
            .into_iter()
            .filter_map(|n| n)
            .reduce(|a, n| r.max(a, n, |n| n.value(heap)))
            .unwrap();

        if max != node {
            heap.swap(node, max, r);
            yield VHeap::new(&heap, Some((node.index, max.index)))
                .now_heapifying(max.index)
                .expl(format!(
                        "Swapped because {} > {}",
                        node.value(heap), max.value(heap)
                ))
                .expl(format!("Next: sift heap at {}", max.value(heap)));
            for_coro!(y in heapify(max, heap, r) => yield y);
        }
    }
}
