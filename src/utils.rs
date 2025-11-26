use leptos::prelude::*;

use std::pin::Pin;
use std::ops::{Range, CoroutineState, Coroutine};

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

impl IsVew for Vew {
    fn list(&self) -> &[usize] {
        self.inner.list()
    }

    fn swapped(&self) -> Option<(usize, usize)> {
        self.inner.swapped()
    }

    fn into_view(&self) -> AnyView {
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
    fn swapped(&self) -> Option<(usize, usize)>;
    fn list(&self) -> &[usize];
    fn into_view(&self) -> AnyView;
}

fn list_into_view(
    spadding: usize,
    epadding: usize,
    s: &[usize],
    swapped: Option<(usize, usize)>,
    special: Option<usize>, // Pivot for quicksort
)
    -> impl IntoView + use<>
{
    let s = s.to_owned();
    let width = (spadding + epadding + s.len()) as f32 * 1.7;
    let width_str = format!("width:{width}em");
    view! {
        <table class="array" style=width_str>
            <tr>
            {move || (0..spadding).map(|_| {
                view! {
                    <td class="elem pad">""</td>
                }
            }).collect_view()}
            {move || s.iter().copied().enumerate().map(|(i, v)| {
                let swp = match swapped {
                    Some((a, b)) if i == a || i == b => " swp",
                    _ => "",
                };

                let special = if Some(i) == special {
                    " spc"
                } else {
                    ""
                };

                let class = format!("elem{}{}", swp, special);

                view! {
                    <td class=class>{v}</td>
                }
            }).collect_view()}
            {move || (0..epadding).map(|_| {
                view! {
                    <td class="elem pad">""</td>
                }
            }).collect_view()}
            </tr>
        </table>
    }
}

pub struct VList(Box<[usize]>);

impl IsVew for VList {
    fn swapped(&self) -> Option<(usize, usize)> {
        None
    }

    fn list(&self) -> &[usize] {
        &self.0
    }

    fn into_view(&self) -> AnyView {
        list_into_view(0, 0, &self.0, None, None).into_any()
    }
}

pub struct VHeap {
    heap: Box<[usize]>,
    n: usize,
    swapped: Option<(usize, usize)>,
}

impl VHeap {
    pub fn new(heap: &Heap<'_>, swapped: Option<(usize, usize)>) -> Self {
        VHeap {
            heap: Box::from(heap.repr()),
            n: heap.nodes(),
            swapped,
        }
    }
}

impl IsVew for VHeap {
    fn swapped(&self) -> Option<(usize, usize)> {
        self.swapped
    }

    fn list(&self) -> &[usize] {
        &self.heap
    }

    fn into_view(&self) -> AnyView {
        let swapped = self.swapped; // captured by closure
        let heap_repr = self.heap.clone(); // for closure
        let heap_len = self.heap.len(); // for closure
        let actual_heap_len = self.n;

        let listview = list_into_view(0, 0, &self.heap, swapped, None).into_any();

        let font_size = "0.8em";
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
            <div style="border: 2px solid #888; border-radius: 0.3em; padding: 0.2em; justify-content: center; display: flex; flex-direction: column; margin-bottom: 1.5em">
                <div style="justify-content: center; display: grid">
                    {listview}
                </div>
                <div style="justify-content: center; display: grid">
                    <svg xmlns="http://www.w3.org/2000/svg" width=width height=height>
                        <defs>
                          <linearGradient id="member" gradientTransform="rotate(90)">
                            <stop offset="80%" stop-color="#3f6f4f" />
                            <stop offset="90%" stop-color="#1f6f2f" />
                          </linearGradient>
                          <linearGradient id="abandoned" gradientTransform="rotate(90)">
                            <stop offset="50%" stop-color="#ffefbf" />
                            <stop offset="70%" stop-color="#ffee99" />
                            <stop offset="95%" stop-color="#ffd060" />
                          </linearGradient>
                          <linearGradient id="swappejd" gradientTransform="rotate(90)">
                            <stop offset="50%" stop-color="#bfefff" />
                            <stop offset="70%" stop-color="#99eeff" />
                            <stop offset="95%" stop-color="#60d0ff" />
                          </linearGradient>
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
            </div>
        }.into_any()
    }
}

impl Into<Vew> for VHeap {
    fn into(self) -> Vew {
        Vew {
            inner: Box::new(self)
        }
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
    pub fn new(list: Box<[usize]>, swapped: Option<(usize, usize)>, expl: String) -> Self {
        VQuick {
            list,
            layers: Vec::new(),
            swapped,
            expl
        }
    }

    pub fn layer(mut self, r: Range<usize>, p: Option<usize>) -> Self {
        self.layers.push(VQuickLayer { r, p });
        self
    }
}

impl IsVew for VQuick {
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
            <div style="border: 2px solid #888; border-radius: 0.3em; padding: 0.2em; justify-content: center; display: flex; flex-direction: column; margin-bottom: 1.5em">
                <div style="justify-content: center; display: grid">
                    {move || {
                        let mut swapped = swapped;
                        let mut expl = Some(expl.clone());
                        layers.iter().map(|l| {
                            view! {
                                {list_into_view(
                                    l.r.start,
                                    list.len() - l.r.end,
                                    &list[l.r.clone()],

                                    // Show swapped elements only for first layer.
                                    swapped.take(),

                                    l.p.map(|p| p - l.r.start),
                                )}
                                {expl.take().map(|expl| view! {
                                    <p class="expl">{expl}</p>
                                })}
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
        }.into_any()
    }
}

impl Into<Vew> for VQuick {
    fn into(self) -> Vew {
        Vew {
            inner: Box::new(self)
        }
    }
}
