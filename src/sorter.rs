pub mod heap;

use crate::for_coro;
use crate::utils::{
    //O, Perf,
    RecorderState,
    Coro,
    Vew, VList, VHeap, VQuick
};

use leptos::prelude::{Write, WriteSignal, Read, ReadSignal, Set, signal};

use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

pub type List = Box<[usize]>;
pub type SortingCoro = Pin<Box<dyn Coroutine<(), Yield = Vew, Return = ()>>>;

pub struct RecorderCallGuard<'a> {
    name: &'static str,
    recorder: &'a Recorder,
}

impl Drop for RecorderCallGuard<'_> {
    fn drop(&mut self) {
        self.recorder.0.write().f_pop();
    }
}

#[derive(Copy, Clone)]
pub struct Recorder(WriteSignal<RecorderState>, pub ReadSignal<RecorderState>);

impl Recorder {
    pub fn new() -> Self {
        let (r, w) = signal(RecorderState::default());
        Self(w, r)
    }

    pub fn reset(&self) {
        self.0.set(Default::default());
    }

    pub fn f(&self, n: &'static str) -> RecorderCallGuard<'_> {
        self.0.write().f_push(n);
        RecorderCallGuard { name: n, recorder: self }
    }

    pub fn lt(&self, a: usize, b: usize) -> bool {
        self.0.write().lt(a, b)
    }

    pub fn gt(&self, a: usize, b: usize) -> bool {
        self.0.write().gt(a, b)
    }

    pub fn min(&self, a: usize, b: usize, by: impl Fn(usize) -> usize) -> usize {
        self.0.write().min(a, b, by)
    }

    pub fn max<T: Copy>(&self, a: T, b: T, by: impl Fn(T) -> usize) -> T {
        self.0.write().max(a, b, by)
    }

    pub fn swap(&self, x: &mut [usize], a: usize, b: usize) {
        self.0.write().swap(x, a, b)
    }

    pub fn count_comparisons(&self) -> usize {
        self.1.read().count_comparisons()
    }

    pub fn count_swaps(&self) -> usize {
        self.1.read().count_swaps()
    }

    pub fn count_calls(&self) -> usize {
        self.1.read().count_calls()
    }
}

pub const ALGORITHMS: &[Algorithm] = &[
    // First is default
    Algorithm::Insertion,

    Algorithm::Quick,
    Algorithm::Selection,
    Algorithm::Bubble,
    Algorithm::Heap,
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
    //Drift,
}

impl Algorithm {
    pub fn func(&self) -> fn(Box<[usize]>, Recorder) -> SortingCoro {
        match self {
            Algorithm::Bubble => |v, r| Box::pin(bubble(v, r)),
            Algorithm::Selection => |v, r| Box::pin(selection(v, r)),
            Algorithm::Insertion => |v, r| Box::pin(insertion(v, r)),
            Algorithm::Heap => |v, r| Box::pin(heap(v, r)),
            Algorithm::Quick => |v, r| Box::pin(quicksort(v, r)),
        }
    }

    // pub fn perf(&self) -> Perf {
    //     match self {
    //         Algorithm::Bubble => Perf::new(O::N2, O::N2, O::N, O::C),
    //         Algorithm::Selection => Perf::new(O::N2, O::N2, O::N2, O::N2),
    //         Algorithm::Insertion => Perf::new(O::N2, O::N2, O::N, O::C),
    //         Algorithm::Heap => Perf::new(O::NLogN, O::NLogN, O::NLogN, O::NLogN),
    //         Algorithm::Quick => Perf::new(O::N2, O::N2, O::NLogN, O::NLogN),
    //     }
    // }
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

pub fn bubble(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
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

pub fn selection(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("selection");
        yield VList::new(&x).into();

        for i in 0..(x.len() - 1) {
            let min = (i..x.len())
                .reduce(|a, v| r.min(a, v, |v| x[v]))
                .unwrap();


            r.swap(&mut x, i, min);
            yield VList::new(&x)
                .swapped(i, min)
                .special(min)
                .into();
        }
    }
}

pub fn insertion(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
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

pub fn heap(mut x: List, r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("heapsort");
        let mut h = heap::Heap::new(&mut x);
        yield VHeap::new(&h, None).into();

        for_coro!(view in build_heap(&mut h, r) =>
            yield view
        );

        for i in (1..h.nodes()).rev() {
            h.swap(heap::Node::of(0, &h), heap::Node::of(i, &h), r);
            yield VHeap::new(&h, Some((0, i))).into();

            h.abandon(1);
            for_coro!(y in heapify(h.root(), &mut h, r) => yield y);
        }
    }
}

fn build_heap(heap: &mut heap::Heap<'_>, r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("build_heap");
        let k = heap.nodes() / 2;
        for i in (0..k).rev() {
            let n = heap::Node::of(i, heap);
            for_coro!(y in heapify(n, heap, r) => yield y);
        }
    }
}

// Based on Introduction to Algorithms, 6.2
fn heapify(
    node: heap::Node,
    heap: &mut heap::Heap<'_>,
    mut r: Recorder,
) -> impl Coroutine<(), Yield = Vew, Return = ()>
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
                .into();
            for_coro!(y in heapify(max, heap, r) => yield y);
        }
    }
}

pub fn quicksort(mut x: List, r: Recorder) -> impl Coroutine<(), Yield = Vew, Return = ()> {
    #[coroutine] static move || {
        let l = x.len();
        yield VQuick::new(x.clone(), None, "").layer(0..x.len(), None).into();
        for_coro!(y in _quicksort(&mut x, 0, l, r) =>
            yield y.layer(0..l, None).into()
        );
    }
}

fn _quicksort<'a>(
    x: &'a mut [usize], s: usize, e: usize, mut r: Recorder
) -> impl Coroutine<(), Yield = VQuick, Return = ()>
{
    #[coroutine] static move || {
        let _g = r.f("quicksort");
        if x[s..e].len() <= 1 {
            return;
        }

        let mut cloned: Box<[usize]> = Box::from(&*x);
        let pivot = s + for_coro!(
            PartitionView { expl, swapped, pivot } in qspartition(&mut x[s..e], r) => {
                if let Some((s1, s2)) = swapped {
                    cloned.swap(s + s1, s + s2);
                }
                yield VQuick::new(cloned.clone(), swapped, expl)
                    .layer(s..e, Some(s + pivot));
            }
        );

        for_coro!(y in _quicksort(x, s, pivot, r) =>
            yield y.layer(s..e, Some(pivot))
        );
        for_coro!(y in _quicksort(x, pivot + 1, e, r) =>
            yield y.layer(s..e, Some(pivot))
        );
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


struct PartitionView {
    expl: String,
    swapped: Option<(usize, usize)>,
    pivot: usize,
}

impl PartitionView {
    pub fn new(expl: &str, s1: usize, s2: usize, p: usize) -> Self {
        PartitionView {
            expl: expl.to_owned(),
            swapped: Some((s1, s2)),
            pivot: p,
        }
    }

    pub fn new2(expl: &str, p: usize) -> Self {
        PartitionView {
            expl: expl.to_owned(),
            swapped: None,
            pivot: p,
        }
    }
}

fn qspartition<'a>(x: &'a mut [usize], mut r: Recorder) -> impl Coroutine<(), Yield = PartitionView, Return = usize> {
    #[coroutine] static move || {
        let _g = r.f("partition");
        let pivot = x.len() / 2;
        yield PartitionView::new2("Chose a pivot", pivot);
        r.swap(x, pivot, x.len() - 1);
        yield PartitionView::new("Moved pivot to end", pivot, x.len() - 1, pivot);

        let mut i = 0;
        for j in 0..x.len() - 1 {
            if r.lt(x[j], x[x.len() - 1]) {
                r.swap(x, i, j);
                yield PartitionView::new("Partitioning", i, j, pivot);
                i += 1;
            }
        }

        r.swap(x, i, x.len() - 1);
        yield PartitionView::new("Moved pivot back", i, x.len() - 1, pivot);
        i
    }
}
