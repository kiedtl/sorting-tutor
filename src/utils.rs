use leptos::prelude::*;
use leptos::html;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Range, CoroutineState, Coroutine};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicUsize};

use crate::sorter::heap::{Heap, Node};

#[macro_export]
macro_rules! for_coro {
    ($value:pat in $coro:expr => $b:expr) => {
        {
            let mut coro = Box::pin($coro);
            loop {
                match coro.as_mut().resume(()) {
                    CoroutineState::Yielded($value) => $b,
                    CoroutineState::Complete(ret) => break ret,
                }
            }
        }
    }
}

pub struct Coro<Y> {
    func: Pin<Box<dyn Coroutine<(), Yield = Y, Return = ()>>>,
    done: bool,
}

impl<Y> Coro<Y> {
    pub fn new(s: Pin<Box<dyn Coroutine<(), Yield = Y, Return = ()>>>) -> Self {
        Self {
            func: s,
            done: false
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
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

// #[derive(Copy, Clone, Debug)]
// pub enum O {
//     C, // Constant-time, O(1)
//     N,
//     LogN,
//     NLogN,
//     N2,
// }

// pub struct Perf {
//     cmp: (O, O),
//     swp: (O, O),
// }

// impl Perf {
//     pub fn new(worst_cmp: O, worst_swp: O, best_cmp: O, best_swp: O) -> Self {
//         Perf {
//             cmp: (worst_cmp, best_cmp),
//             swp: (worst_swp, best_swp),
//         }
//     }
// }

#[derive(Clone, Debug, Default)]
pub struct RecorderState {
    // comparisons: Arc<AtomicUsize>,
    // swaps: Arc<AtomicUsize>,
    comparisons: usize,
    swaps: usize,
    stack: Vec<&'static str>,
    calls: usize,
}

impl RecorderState {
    pub fn f_push(&mut self, name: &'static str) {
        self.calls += 1;
        self.stack.push(name);
    }

    pub fn f_pop(&mut self) -> Option<&'static str> {
        self.stack.pop()
    }

    pub fn lt(&mut self, a: usize, b: usize) -> bool {
        //self.comparisons.fetch_add(1, Ordering::Relaxed);
        self.comparisons += 1;
        a < b
    }

    pub fn gt(&mut self, a: usize, b: usize) -> bool {
        //self.comparisons.fetch_add(1, Ordering::Relaxed);
        self.comparisons += 1;
        a > b
    }

    pub fn max<T: Copy>(&mut self, a: T, b: T, by: impl Fn(T) -> usize) -> T {
        self.comparisons += 1;
        if by(a) >= by(b) {
            a
        } else {
            b
        }
    }

    pub fn min(&mut self, a: usize, b: usize, by: impl Fn(usize) -> usize) -> usize {
        self.comparisons += 1;
        if by(a) <= by(b) {
            a
        } else {
            b
        }
    }

    pub fn swap(&mut self, x: &mut [usize], a: usize, b: usize) {
        //self.swaps.fetch_add(1, Ordering::Relaxed);
        self.swaps += 1;
        x.swap(a, b);
    }

    pub fn count_comparisons(&self) -> usize {
        //self.comparisons.load(Ordering::Relaxed)
        self.comparisons
    }

    pub fn count_swaps(&self) -> usize {
        //self.swaps.load(Ordering::Relaxed)
        self.swaps
    }

    pub fn count_calls(&self) -> usize {
        self.calls
    }

    pub fn get_call_stack(&self) -> &[&'static str] {
        &self.stack
    }
}

pub fn size_class(item_len: usize) -> &'static str {
    match item_len {
        00..15 => " z1",
        15..20 => " z2",
        20..30 => " z3",
        30..40 => " z4",
        _ => " z5",
    }
}

// A "snapshot" of progress of a sorting algorithm, at the very least containing
// the partially sorted list and possibly also annotations, tree structures, etc.
// Not to be confused with Leptos' View/IntoView stuff.
pub struct Snapshot {
    inner: Box<dyn IsSnapshot>,
    is_important: bool,
    hash: u64,
}

impl Snapshot {
    pub fn new(inner: Box<dyn IsSnapshot>) -> Self {
        let mut hasher = DefaultHasher::new();
        inner.list().hash(&mut hasher);
        let hash = hasher.finish();

        Snapshot {
            inner,
            is_important: true,
            hash,
        }
    }

    pub fn list(&self) -> &[usize] {
        self.inner.list()
    }

    pub fn swapped(&self) -> Option<(usize, usize)> {
        self.inner.swapped()
    }

    pub fn into_view(&self) -> AnyView {
        self.inner.into_view()
    }

    pub fn is_important(&self) -> bool {
        self.is_important
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn fleeting(mut self) -> Self {
        self.is_important = false;
        self
    }
}

impl From<&[usize]> for Snapshot {
    fn from(f: &[usize]) -> Snapshot {
        // A Box inside a Box made from a borrowed Box. Incredibly wasteful.
        // Snapshot should just be an enum.
        Snapshot::new(Box::new(VList::new(f)))
    }
}

impl From<&Box<[usize]>> for Snapshot {
    fn from(f: &Box<[usize]>) -> Snapshot {
        Snapshot::new(Box::new(VList::new(&f)))
    }
}

pub trait IsSnapshot: Send + Sync {
    fn swapped(&self) -> Option<(usize, usize)>;
    fn list(&self) -> &[usize];
    fn into_view(&self) -> AnyView;

    fn into_vew(self) -> Snapshot where Self: Sized + 'static {
        Snapshot::from(self)
    }
}

impl<T> From<T> for Snapshot
where
    T: IsSnapshot + 'static,
{
    fn from(value: T) -> Snapshot {
        Snapshot::new(Box::new(value))
    }
}

fn list_into_view(
    spadding: usize,
    epadding: usize,
    s: &[usize],
    swapped: Option<(usize, usize)>,
    special: Option<usize>, // Pivot for quicksort
    current: Option<usize>,
)
    -> impl IntoView + use<>
{
    let s = s.to_owned();
    let elem_width_class = size_class(s.len() + spadding + epadding);
    let pad_class = format!("elem pad {elem_width_class}"); 
    let pad_class_cloned = format!("elem pad {elem_width_class}"); 

    // let width_fac = if s.len() > 32 { 1.0 } else { 1.7 };
    // let width = (spadding + epadding + s.len()) as f32 * width_fac;
    // let width_str = format!("width:{width}em");

    view! {
        {move || (0..spadding).map(|_| {
            let c = pad_class.clone();
            view! {
                <div class=c></div>
            }
        }).collect_view()}
        {move || s.iter().copied().enumerate().map(|(i, v)| {
            let swp = match swapped {
                Some((a, b)) if i == a || i == b => " swp",
                _ => "",
            };

            let special = if Some(i) == special { " spc" } else { "" };
            let current = if Some(i) == current { " cur" } else { "" };

            let class = format!("elem{swp}{special}{current}{elem_width_class}");

            view! {
                <div class=class>{v}</div>
            }
        }).collect_view()}
        {move || (0..epadding).map(|_| {
            let c = pad_class_cloned.clone();
            view! {
                <div class=c></div>
            }
        }).collect_view()}
    }
}

pub struct VList {
    list: Box<[usize]>,
    swapped: Option<(usize, usize)>,
    special: Option<usize>,
    current: Option<usize>,
    expl: Option<String>,
}

impl VList {
    pub fn new(x: &[usize]) -> Self {
        VList {
            list: Box::from(x),
            swapped: None,
            special: None,
            current: None,
            expl: None,
        }
    }

    pub fn swapped(mut self, s1: usize, s2: usize) -> Self {
        self.swapped = Some((s1, s2));
        self
    }

    pub fn special(mut self, spc: usize) -> Self {
        self.special = Some(spc);
        self
    }

    pub fn current(mut self, spc: usize) -> Self {
        self.current = Some(spc);
        self
    }

    pub fn expl(mut self, expl: String) -> Self {
        self.expl = Some(expl);
        self
    }
}

impl IsSnapshot for VList {
    fn swapped(&self) -> Option<(usize, usize)> {
        self.swapped
    }

    fn list(&self) -> &[usize] {
        &self.list
    }

    fn into_view(&self) -> AnyView {
        view! {
            <div class="solo-group">
                <div class="enclosure">
                    {list_into_view(0, 0, &self.list, self.swapped, self.special, self.current)}
                </div>
            </div>
        }.into_any()
    }
}

pub struct VHeap {
    heap: Box<[usize]>,
    n: usize,
    swapped: Option<(usize, usize)>,
    heapifying: Option<usize>,
    expl: Vec<String>,
}

impl VHeap {
    pub fn new(heap: &Heap<'_>, swapped: Option<(usize, usize)>) -> Self {
        VHeap {
            heap: Box::from(heap.repr()),
            n: heap.nodes(),
            swapped,
            heapifying: None,
            expl: Vec::new(),
        }
    }

    pub fn expl(mut self, s: impl Into<String>) -> Self {
        self.expl.push(s.into());
        self
    }

    pub fn now_heapifying(mut self, i: usize) -> Self {
        self.heapifying = Some(i);
        self
    }
}

impl IsSnapshot for VHeap {
    fn swapped(&self) -> Option<(usize, usize)> {
        self.swapped
    }

    fn list(&self) -> &[usize] {
        &self.heap
    }

    fn into_view(&self) -> AnyView {
        let swapped = self.swapped; // captured by closure
        let heapifying = self.heapifying; // for closure
        let heap_repr = self.heap.clone(); // for closure
        let heap_len = self.heap.len(); // for closure
        let expl = self.expl.clone(); // captured by closure
        let actual_heap_len = self.n;

        let listview = list_into_view(0, 0, &self.heap, swapped, None, None).into_any();

        let font_size = "0.75em";
        let bw = 20;
        let bh = 20;
        let bdr = 2;
        let level_padding = 8;
        let level_spacing = bh + level_padding;
        let margin_top = 8;

        let bottom_level = Node::of_unchecked(heap_len - 1).level();
        let bottom_level_occupants = 1 << bottom_level;
        let width = bottom_level_occupants * (bw + 4 + (2 * bdr)); // 4 pixel spacing + 2 borders
        let height = 0
            + margin_top
            + ((bottom_level + 1) * level_spacing);

        let get_pos_for = move |i| {
            let node = Node::of_unchecked(i);
            let occupants = 1 << node.level();
            let index_at_level = i - (occupants - 1);

            let x = (index_at_level + 1) * (width / (occupants + 1));
            let y = margin_top + level_spacing * node.level();

            (x, y)
        };

        view! {
            <div class="group">
                <div class="enclosure">
                    {listview}
                </div>
                <div class="enclosure">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        //width=width height=height
                        width="100%" height="100%"
                        style=move || format!("max-width: {width}px; height: {height}px")
                        viewBox=move || format!("0 0 {width} {height}")
                        preserveAspectRatio="xMidYMid meet"
                    >
                        <defs>
                          <linearGradient id="member" gradientTransform="rotate(90)">
                            <stop offset="80%" stop-color="#3f6f4f" />
                            <stop offset="90%" stop-color="#1f6f2f" />
                          </linearGradient>
                          <linearGradient id="abandoned" gradientTransform="rotate(90)">
                            <stop offset="50%" stop-color="#ffefbf" />
                            <stop offset="70%" stop-color="#ffee99" />
                            <stop offset="99%" stop-color="#ffd060" />
                          </linearGradient>
                          // <linearGradient id="swapped" gradientTransform="rotate(90)">
                          //   <stop offset="50%" stop-color="#bfefff" />
                          //   <stop offset="70%" stop-color="#99eeff" />
                          //   <stop offset="95%" stop-color="#60d0ff" />
                          // </linearGradient>
                          <linearGradient id="swapped">
                            <stop offset="100%" stop-color="#2f4f8f" />
                          </linearGradient>
                        </defs>

                        {move || (1..actual_heap_len) // Skip root
                            .map(|i| {
                                let parent = (i - (if i % 2 == 1 { 1 } else { 2 })) / 2;

                                let (cx, cy) = get_pos_for(i);
                                let (px, py) = get_pos_for(parent);

                                view! {
                                    <line
                                        x1={px + bw / 2}
                                        y1={py + bh / 2}
                                        x2={cx + bw / 2}
                                        y2={cy + bh / 2}
                                        stroke="black"
                                        stroke-width={bdr}
                                    />
                                }
                            })
                            .collect_view()
                        }

                        {move || heapifying.map(|heapifying| {
                            let (x, y) = get_pos_for(heapifying);
                            view! {
                                <circle
                                    cx={x + bw / 2}
                                    cy={y + bw / 2}
                                    r={bw * 80 / 100}
                                    stroke="#af4f4f"
                                    stroke-width=2
                                    fill="#00000000"
                                />
                            }
                        })}

                        {move || heap_repr
                            .iter()
                            .enumerate()
                            .map(|(i, value)| {
                                let (x, y) = get_pos_for(i);
                                let value = value.to_string();
                                let (tfill, fill) = if i >= actual_heap_len {
                                    ("black", "url('#abandoned')")
                                } else if matches!(swapped, Some((a, b)) if i == a || i == b) {
                                    ("white", "url('#swapped')")
                                } else {
                                    ("white", "url('#member')")
                                };
                                view! {
                                    <rect
                                        x=x y=y rx=4
                                        fill=fill
                                        stroke="black"
                                        stroke-width=2
                                        width=bw height=bh
                                    />
                                    <text
                                        x={x + bw / 8}
                                        y={y + bh / 4 * 3}
                                        font-family="monospace"
                                        font-size={font_size}
                                        fill=tfill
                                    >
                                        {value}
                                    </text>
                                }
                            })
                            .collect_view()
                        }
                    </svg>
                </div>
                // <div class="enclosure">
                    {expl
                        .into_iter()
                        .map(|expl|
                            html::p().class("expl").child(expl)
                        )
                        .collect_view()
                    }
                // </div>
            </div>
        }.into_any()
    }
}

#[derive(Clone)]
pub struct VQuickLayer {
    r: Range<usize>,
    p: Option<usize>,
}

pub struct VQuick {
    list: Box<[usize]>,
    layers: Vec<VQuickLayer>,
    swapped: Option<(usize, usize)>,
    expl: String,
}

impl VQuick {
    pub fn new(list: Box<[usize]>, swapped: Option<(usize, usize)>, expl: impl Into<String>) -> Self {
        VQuick {
            list,
            layers: Vec::new(),
            swapped,
            expl: expl.into(),
        }
    }

    pub fn layer(mut self, r: Range<usize>, p: Option<usize>) -> Self {
        self.layers.push(VQuickLayer { r, p });
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

    fn into_view(&self) -> AnyView {
        let swapped = self.swapped; // captured by closure
        let expl = self.expl.clone(); // captured by closure

        let list = self.list.clone();
        let layers = self.layers.clone();

        view! {
            <div class="group">
                {move || {
                    let mut swapped = swapped;
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
                                    None,
                                )}
                            </div>
                            <div class="enclosure">
                                {expl.take().map(|expl| view! {
                                    <p class="expl">{expl}</p>
                                })}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        }.into_any()
    }
}
