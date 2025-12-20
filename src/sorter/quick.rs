use leptos::prelude::*;
use leptos::html;

use crate::for_coro;
use crate::sorter::List;
use crate::recorder::Recorder;
use crate::utils::{ Coro, IsSnapshot, Snapshot, list_into_view };

use std::pin::Pin;
use std::ops::{Range, CoroutineState, Coroutine};

pub fn sort(mut x: List, r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let l = x.len();
        yield VQuick::new(x.clone(), None, "").layer(0..x.len(), None).into();
        for_coro!(y in _sort(&mut x, 0, l, r) =>
            yield y.layer(0..l, None).into()
        );
    }
}

fn _sort<'a>(
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
            PartitionView { expl, swapped, pivot, fleeting, current } in partition(&mut x[s..e], r) => {
                if let Some((s1, s2)) = swapped {
                    cloned.swap(s + s1, s + s2);
                }
                yield VQuick::new(cloned.clone(), swapped, expl)
                    .layer(s..e, Some(s + pivot))
                    .current(current)
                    .set_fleeting(fleeting);
            }
        );

        for_coro!(y in _sort(x, s, pivot, r) =>
            yield y.layer(s..e, Some(pivot))
        );
        for_coro!(y in _sort(x, pivot + 1, e, r) =>
            yield y.layer(s..e, Some(pivot))
        );
    }
}

struct PartitionView {
    expl: String,
    swapped: Option<(usize, usize)>,
    pivot: usize,
    fleeting: bool,
    current: Option<usize>,
}

impl PartitionView {
    pub fn new(expl: &str, s1: usize, s2: usize, p: usize) -> Self {
        PartitionView {
            expl: expl.to_owned(),
            swapped: Some((s1, s2)),
            pivot: p,
            fleeting: false,
            current: None,
        }
    }

    pub fn new2(expl: &str, p: usize) -> Self {
        PartitionView {
            expl: expl.to_owned(),
            swapped: None,
            pivot: p,
            fleeting: false,
            current: None,
        }
    }

    pub fn fleeting(mut self) -> Self {
        self.fleeting = true;
        self
    }

    pub fn current(mut self, v: usize) -> Self {
        self.current = Some(v);
        self
    }
}

fn partition<'a>(x: &'a mut [usize], mut r: Recorder) -> impl Coroutine<(), Yield = PartitionView, Return = usize> {
    #[coroutine] static move || {
        let _g = r.f("partition");
        let pivot = x.len() / 2;
        yield PartitionView::new2("Chose a pivot", pivot);
        r.swap(x, pivot, x.len() - 1);
        yield PartitionView::new("Moved pivot to end", pivot, x.len() - 1, x.len() - 1);

        let mut i = 0;
        for j in 0..x.len() - 1 {
            if r.lt(x[j], x[x.len() - 1]) {
                if i == j {
                    yield PartitionView::new2("Partitioning", x.len() - 1)
                        .fleeting()
                        .current(j);
                } else {
                    r.swap(x, i, j);
                    yield PartitionView::new("Partitioning", i, j, x.len() - 1);
                }
                i += 1;
            } else {
                yield PartitionView::new2("Partitioning", x.len() - 1)
                    .fleeting()
                    .current(j);
            }
        }

        r.swap(x, i, x.len() - 1);
        yield PartitionView::new("Moved pivot back", i, x.len() - 1, i);
        i
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

//     let pivot = partition(x);
//     yield Box::from(&mut *x);

//     for view in Box::new(_quicksort(&mut x[..pivot])) {
//         yield view;
//     }

//     for view in Box::new(_quicksort(&mut x[pivot + 1..])) {
//         yield view;
//     }
// }

#[derive(Clone)]
pub struct VQuickLayer {
    r: Range<usize>,
    p: Option<usize>,
}

pub struct VQuick {
    list: Box<[usize]>,
    layers: Vec<VQuickLayer>,
    swapped: Option<(usize, usize)>,
    current: Option<usize>,
    expl: String,
    fleeting: bool,
}

impl VQuick {
    pub fn new(list: Box<[usize]>, swapped: Option<(usize, usize)>, expl: impl Into<String>) -> Self {
        VQuick {
            list,
            layers: Vec::new(),
            swapped,
            current: None,
            expl: expl.into(),
            fleeting: false,
        }
    }

    pub fn layer(mut self, r: Range<usize>, p: Option<usize>) -> Self {
        self.layers.push(VQuickLayer { r, p });
        self
    }

    pub fn set_fleeting(mut self, value: bool) -> Self {
        self.fleeting = value;
        self
    }

    pub fn current(mut self, current: Option<usize>) -> Self {
        self.current = current;
        self
    }
}

impl IsSnapshot for VQuick {
    fn swapped(&self) -> Option<(usize, usize)> {
        self.swapped
    }

    fn list(&self) -> &[usize] {
        &self.list
    }

    fn is_fleeting(&self) -> bool {
        self.fleeting
    }

    fn into_view(&self) -> AnyView {
        let swapped = self.swapped; // captured by closure
        let current = self.current; // captured by closure
        let expl = self.expl.clone(); // captured by closure

        let list = self.list.clone();
        let layers = self.layers.clone();

        view! {
            <div class="group">
                {move || {
                    let mut swapped = swapped;
                    let mut current = current;
                    let mut expl = (!expl.is_empty()).then(|| expl.clone());
                    layers.iter().map(|l| {
                        view! {
                            <div class="enclosure">
                                {list_into_view(
                                    l.r.start,
                                    list.len() - l.r.end,
                                    &list[l.r.clone()],

                                    // Show swapped elements only for first layer.
                                    swapped.take(),

                                    l.p.map(|p| p - l.r.start),

                                    // Show current element only for first layer.
                                    current.take(),
                                )}
                            </div>
                            {expl.take().map(|expl| view! {
                                <div class="enclosure">
                                        <p class="expl">{expl}</p>
                                </div>
                            })}
                        }
                    }).collect_view()
                }}
            </div>
        }.into_any()
    }
}
