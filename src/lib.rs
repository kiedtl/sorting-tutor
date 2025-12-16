#![allow(dead_code)]
#![allow(unused)]

#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(gen_blocks)]

mod utils;
mod sorter;

use crate::utils::{Vew, IsVew, Coro};
use crate::sorter::{Recorder, Algorithm, ALGORITHMS, List};

use std::pin::Pin;
use std::ops::Coroutine;
use std::ops::Deref;

use gloo::timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use rand::prelude::*;
use reactive_stores::{Subfield, Store};
use leptos::tachys::reactive_graph::bind::IntoSplitSignal;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (::web_sys::console::log_1(&format!($($t)*).into()))
}

fn gen_values(
    rng: &mut ThreadRng,
    generation: DataGenerationOption,
    shape: DataShapeOption,
    size: usize,
) -> Box<[usize]> {
    let mut v: Vec<usize> = match generation {
        DataGenerationOption::Random => (0..size).map(|_| rng.random_range(10..99)).collect(),
        DataGenerationOption::Sequential => (1..=size).collect(),
    };

    let prepare_divide_list = |v: &mut Vec<usize>| {
        v.sort();
        for i in 0..(size / 2) {
            v.swap(i, i * 2);
        }
    };

    match shape {
        DataShapeOption::Shuffled => v.shuffle(rng),
        DataShapeOption::Ascending => v.sort(),
        DataShapeOption::Descending => v.sort_by(|a, b| b.cmp(a)),
        DataShapeOption::Mountain => {
            prepare_divide_list(&mut v);
            (&mut v[..size / 2]).sort();
            (&mut v[size / 2..]).sort_by(|a, b| b.cmp(a));
        },
        DataShapeOption::Valley => {
            prepare_divide_list(&mut v);
            (&mut v[size / 2..]).sort();
            (&mut v[..size / 2]).sort_by(|a, b| b.cmp(a));
        },
    }

    v.into_boxed_slice()
}

fn run_once(
    sorter_w: WriteSignal<Coro<Vew>, LocalStorage>,
    history_w: WriteSignal<Vec<Vew>>,
) {
    match sorter_w.write().next() {
        Some(view) => history_w.write().push(view),
        None => (),
    }
}

#[derive(Store, Copy, Clone,)]
struct VisualOptions {
    grid: bool,
    bars: bool,
    values: bool,
    history: bool,
}

impl Default for VisualOptions {
    fn default() -> Self {
        Self {
            grid: true,
            bars: true,
            values: true,
            history: true,
        }
    }
}

#[derive(Store, Copy, Clone)]
struct DataOptions {
    shape: DataShapeOption,
    generation: DataGenerationOption,
    size: usize,
}

impl Default for DataOptions {
    fn default() -> Self {
        Self {
            generation: Default::default(),
            shape: Default::default(),
            // Default size is power of two minus two -- good for heapsort, since it
            // means an "almost-full" tree
            size: 14,
        }
    }
}

macro_rules! enum_selection {
    (
        $(#[$toplevelmetas:meta])*
        enum $enum_name:ident {
            $(
                $(#[$m:meta])*
                $field:ident = $value:expr,
            )+
        }
    ) => {
        $(#[$toplevelmetas])*
        enum $enum_name {
            $($(#[$m])* $field,)*
        }

        impl $enum_name {
            pub fn selection_option_view() -> impl IntoView {
                use leptos::html::option;
                (
                    $(
                        option().attr("value", $value).child($value),
                    )+
                )
            }
        }

        impl From<&'_ str> for $enum_name {
            fn from(s: &str) -> Self {
                [ $(($enum_name::$field, $value),)+ ]
                    .iter()
                    .copied()
                    .find(|(inv, inv_str)| *inv_str == s)
                    .map(|(inv, _)| inv)
                    .unwrap_or($enum_name::default())
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(
                        $enum_name::$field => $value,
                    )+
                })
            }
        }
    }
}

enum_selection! {
    #[derive(Copy, Clone, Default)]
    enum DataGenerationOption {
        #[default]
        Sequential = "sequential",
        Random = "random",
    }
}

enum_selection! {
    #[derive(Copy, Clone, Default)]
    enum DataShapeOption {
        #[default]
        Shuffled = "shuffled",
        Ascending = "ascending",
        Descending = "descending",
        Mountain = "mountain",
        Valley = "valley",
    }
}

#[component]
fn VisualControlCheckbox(
    text: &'static str,
    binding: Subfield<Store<VisualOptions>, VisualOptions, bool>,
    disabled: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl IntoView
{
    use leptos::attr::Checked;
    use leptos::html::{tr, td, label, input};
    tr()
        .child(
            td().child(
                label()
                    .class(move || if disabled() { "strike" } else { "" })
                    .attr("for", text)
                    .child(text)
            )
        )
        .child(
            td()
                .attr("colspan", 2)
                .class("tiny")
                .child(
                    input()
                        .attr("type", "checkbox")
                        .attr("name", text)
                        .attr("disabled", disabled)
                        .bind(Checked, binding)
                )
        )
}

#[component]
fn Control(
    rng: WriteSignal<ThreadRng, LocalStorage>,
    vopts: Store<VisualOptions>,
    algo_r: ReadSignal<Algorithm>,
    algo_w: WriteSignal<Algorithm>,
    history_r: ReadSignal<Vec<Vew>>,
    history_w: WriteSignal<Vec<Vew>>,
    sorter_w: WriteSignal<Coro<Vew>, LocalStorage>,
    values_w: WriteSignal<Box<[usize]>>,
    dopts: Store<DataOptions>,
    recorder: Recorder,
) -> impl IntoView
{
    let (running_r, running_w) = signal(false);
    let (delay_r, delay_w) = signal(60);

    view! {
        <div class="card">
            <h3 class="card-title">"Control"</h3>
            <hr class="bhr" />
            <button
                on:click=move |_| {
                    if running_r.get() {
                        running_w.set(false);
                    } else {
                        running_w.set(true);
                        spawn_local(async move {
                            while let Some(view) = sorter_w.write_untracked().next() && running_r.get_untracked() {
                                history_w.write().push(view);
                                TimeoutFuture::new(delay_r.get_untracked() as u32).await;
                            }
                            running_w.set(false);
                        });
                    }
                }
            >
                {move || if running_r.get() { "Stop" } else { "Start" }}
            </button>
            <button
                on:click=move |_| run_once(sorter_w, history_w)
            >
            "Step"
            </button>
        </div>
        <div class="card">
            <h3 class="card-title">"Settings"</h3>
            <hr class="bhr" />
            <table class="flat">
                <thead>
                    <th colspan=2>"Algorithm"</th>
                </thead>
                <tbody>
                <tr>
                    <td>
                        <label>"Sorter"</label>
                    </td>
                    <td colspan="2">
                        <select
                            on:change:target=move |ev| {
                                let v = ev.target().value();
                                algo_w.set(
                                    ALGORITHMS
                                        .iter()
                                        .copied()
                                        .find(|al| al.to_string() == v)
                                        .unwrap_or(Algorithm::Insertion)
                                );
                            }
                            prop:value=move || algo_r.get().to_string()
                        >
                            {move || sorter::ALGORITHMS.iter()
                                .enumerate()
                                .map(|(i, algorithm)| {
                                    let s = algorithm.to_string();
                                    view! {
                                        <option value={s}>{s.clone()}</option>
                                    }
                                })
                                .collect_view()
                            }
                        </select>
                    </td>
                </tr>
                <tr>
                    <td>
                        <label for="Delay">Delay</label>
                        <br />
                        <span class="mini m">{move || delay_r.get()}"ms"</span>
                    </td>
                    <td>
                        <input
                            type="range" id="delay" name="Delay" min="0" max="256"
                            value=move || delay_r.get()
                            on:input:target=move |ev| {
                                delay_w.set(ev.target().value().parse().unwrap());
                            }
                        />
                    </td>
                </tr>
                </tbody>
            </table>
            <hr class="fsep" />
            <table class="flat">
                <thead>
                    <th colspan=2>"Data"</th>
                </thead>
                <tbody>
                <tr>
                    <td>
                        <label for="Size">Size</label>
                        <br />
                        <span class="mini m">{move || dopts.size().get()}" nums"</span>
                    </td>
                    <td>
                        <input
                            type="range" id="size" name="Size" min="4" max="48"
                            value=move || dopts.size().get()
                            on:input:target=move |ev| {
                                dopts.size().set(ev.target().value().parse().unwrap());
                            }
                        />
                    </td>
                </tr>
                <tr>
                    <td>
                        <label>"Shape"</label>
                    </td>
                    <td colspan="2">
                        <select
                            on:change:target=move |ev| {
                                dopts.shape().set(DataShapeOption::from(ev.target().value().as_str()));
                            }
                            prop:value=move || dopts.shape().get().to_string()
                        >
                            {DataShapeOption::selection_option_view()}
                        </select>
                    </td>
                </tr>
                <tr>
                    <td>
                        <label>"Gen"</label>
                    </td>
                    <td colspan="2">
                        <select
                            on:change:target=move |ev| {
                                dopts.generation().set(DataGenerationOption::from(ev.target().value().as_str()));
                            }
                            prop:value=move || dopts.generation().get().to_string()
                        >
                            {DataGenerationOption::selection_option_view()}
                        </select>
                    </td>
                </tr>
                <tr>
                    <td colspan="3" style="text-align:right">
                        <button
                            style="width:33%"
                            on:click=move |_| {
                                values_w.set(gen_values(
                                        &mut rng.write_untracked(),
                                        dopts.generation().get(),
                                        dopts.shape().get(),
                                        dopts.size().get()
                                ));
                            }
                        >
                            "Reset"
                        </button>
                    </td>
                </tr>
                </tbody>
            </table>
            <hr class="fsep" />
            <table class="flat">
                <thead>
                    <th colspan="3">"Visualization"</th>
                </thead>
                <tbody>
                    <VisualControlCheckbox text="Show grid" binding=vopts.grid() disabled=|| false />
                    <VisualControlCheckbox text="Show bars" binding=vopts.bars() disabled=|| false />
                    <VisualControlCheckbox text="Show values" binding=vopts.values() disabled=|| false />
                    <VisualControlCheckbox text="Show history" binding=vopts.history()
                        disabled=move || !vopts.values().get()
                    />
                </tbody>
            </table>
        </div>
        <div class="card">
            <h3 class="card-title">"Legend"</h3>
            <hr class="bhr" />
            <p>
                <span class="fake elem">"White"</span>
                "Ordinary elements"
            </p>
            <hr class="fsep" />
            <p>
                <span class="fake elem swp">"Blue"</span>
                "Recently swapped"
            </p>
            <hr class="fsep" />
            <p>
                <span class="fake elem spc">"Gold"</span>
                "Pivots, mins, special items"
            </p>
            <hr class="fsep" />
            <p>
                <span class="fake elem swp spc">"Glue"</span>
                "Recently swapped special items"
            </p>
        </div>
    }
}

#[component]
fn Content(
    dopts: Store<DataOptions>,
    vopts: Store<VisualOptions>,
    history_r: ReadSignal<Vec<Vew>>,
) -> impl IntoView
{
    // let bars = move || history_r.read().last().map(|v| v.list().len()).unwrap_or(0);
    // let bar_width_fac = move || if bars() > 32 { 1. } else { 1.7 };
    // let bar_width = move || bars() as f32 * bar_width_fac();

    let bars = move || {
        let history = history_r.read();
        let bars = history.last().unwrap().list();

        let min = bars.iter().copied().min().unwrap();
        let max = bars.iter().copied().max().unwrap();

        bars
            .iter()
            .copied()
            .map(|v| (v - min) * 100 / (max - min))
            .enumerate()
            .collect::<Vec<_>>()
    };
    let bars_len = move || history_r.read().last().unwrap().list().len();
    let were_bars_swapped = move |i| match history_r.read().last().unwrap().swapped() {
        Some((a, b)) if i == a || i == b => true,
        _ => false,
    };

    let canvas_ref = NodeRef::<html::Canvas>::new();

    Effect::new(move || {
        if let Some(canvas_ref) = canvas_ref.get() {
            let new_dim = match dopts.size().get() {
                0..8 => "100",
                8..18 => "150",
                18..25 => "220",
                25..35 => "320",
                _ => "400",
            };
            canvas_ref.set_attribute("width", new_dim);
            canvas_ref.set_attribute("height", new_dim);
        }
    });

    Effect::new(move || {
        if let Some(canvas_ref) = canvas_ref.get() {
            let context = canvas_ref
              .get_context("2d")
              .unwrap()
              .unwrap()
              .dyn_into::<web_sys::CanvasRenderingContext2d>()
              .unwrap();

            let bars = bars();

            let canv_w = canvas_ref.width() as f64;
            let canv_h = canvas_ref.height() as f64;
            let w = 5.; //(canv_w / bars.len() as f64) * 0.9;
            let h = 5.; //(canv_h / bars.len() as f64) * 0.9;
            let px = ((canv_w - (bars.len() as f64 * w)) / 2.).max(0.);
            let py = ((canv_h - (bars.len() as f64 * h)) / 2.).max(0.);

            context.clear_rect(0., 0., canv_w, canv_h);

            for &(place, value) in &bars {
                let value = value as f64 / 100. * (bars.len() as f64);
                let angle = std::f64::consts::PI / 4.;
                let x = px + place as f64 * (w + 1.);
                let y = py + value        * (w + 1.);
                context.set_line_width(w);
                context.begin_path();
                context.move_to(x, y);
                context.line_to(x + w * angle.cos(), y + h * angle.sin());
                context.stroke();
            }
        }
    });

    view! {
        <Show when=move || vopts.grid().get() >
            <div class="solo-group" style="margin-bottom:1rem">
                <canvas node_ref=canvas_ref width=400 height=400 id="vancas">
                    "Really? A browser that doesn't support canvas? In 2025?"
                </canvas>
            </div>
        </Show>
        <Show when=move || vopts.bars().get() >
            <div class="solo-group">
                <div class="bar-enclosure">
                    <For
                        each=move || bars()
                        key=|&(i, v)| (i, v)
                        let((i, v))
                    >
                        <div
                            class=move || format!(
                                "{} {}",
                                utils::size_class(bars_len()),
                                if were_bars_swapped(i) { "bar swp" } else { "bar" }
                            )
                            style=move || format!("height:{}px", (v * 2 / 3) + 15)
                        >
                        </div>
                    </For>
                </div>
            </div>
        </Show>
        <Show when=move || vopts.values().get() >
            {move || history_r.read().last().unwrap().into_view()}
        </Show>
        <Show when=move || vopts.values().get() && vopts.history().get() >
            <For
                each=move || {
                    let mut found_important = false;
                    history_r
                        .read()
                        .iter()
                        .enumerate()
                        .rev()
                        .skip(1)
                        .filter_map(|(k, item)| {
                            let important = item.is_important();
                            if important || !found_important {
                                if !found_important {
                                    found_important = important;
                                }
                                Some((k, item))
                            } else {
                                None
                            }
                        })
                        .enumerate()
                        .take_while(|(i, (k, _))| *i < 32)
                        .map(|(i, (k, v))| (k, v.hash()))
                        .collect::<Vec<_>>()
                }
                key=|&(k, _)| k
                let((k, _))
            >
            {move || history_r.read()[k].into_view()}
            </For>
        </Show>
    }
}

#[component]
fn Right(
    recorder: Recorder,
) -> impl IntoView
{
    view! {
        <div class="card">
            <h3 class="card-title">"Stats"</h3>
            <hr class="bhr" />
            <table class="flat">
                <thead>
                    <tr>
                        <th colspan="2">
                        "Performance"
                        </th>
                    </tr>
                </thead>
                <tbody>
                <tr>
                    <td>
                        <label>"Comparisons"</label>
                        <button class="tip" popovertarget="comparisons-expl">?</button>
                        <div popover id="comparisons-expl">
                            <h1>"Comparisons"</h1>
                            <hr class="bhr" />
                            "Element-to-element comparisons. Does not include comparisons made when iterating, etc."
                        </div>
                    </td>
                    <td class="tiny m">{move || recorder.count_comparisons()}</td>
                </tr>
                <tr>
                    <td>
                        <label>"Swaps"</label>
                        <button class="tip" popovertarget="swaps-expl">?</button>
                        <div popover id="swaps-expl">
                            <h1>"Element Swaps"</h1>
                            <hr class="bhr" />
                            "Number of times a pair of elements were swapped whilst sorting."
                        </div>
                    </td>
                    <td class="tiny m">{move || recorder.count_swaps()}</td>
                </tr>
                <tr>
                    <td>
                        <label>"Function Calls"</label>
                        <button class="tip" popovertarget="calls-expl">?</button>
                        <div popover id="calls-expl">
                            <h1>"Function Calls"</h1>
                            <hr class="bhr" />
                            <p>
                            "Number of times any function pertaining to the sorting algorithm was called. This includes the initial function call, recursive function calls, calls to significant (non-helper) functions, etc."
                            </p>
                            <p>
                            <strong>"Example"</strong>": For HeapSort, this would include the call to heapsort(), heapsort()'s call to build_heap(), build_heap()'s call and every subsequent recursive call to siftDown(), and so on. It would not include calls to len(), swap(), or other small functions."
                            </p>
                        </div>
                    </td>
                    <td class="tiny m">{move || recorder.count_calls()}</td>
                </tr>
                </tbody>
            </table>
        </div>
        <div class="card">
            <h3 class="card-title">"Stack"</h3>
            <hr class="bhr" />
            <table class="stack">
                {move || recorder.1.read().get_call_stack()
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(i, &s)| view! {
                        <tr>
                            <td class="frame-number m">{i.to_string()}</td>
                            <td class="frame">
                                <code>{s}</code>
                            </td>
                        </tr>
                    })
                    .collect_view()
                }
                {move || (0..(10usize.saturating_sub(recorder.1.read().get_call_stack().len())))
                    .map(|_| view! {
                        <tr>
                            <td class="frame-number empty-frame"></td>
                            <td class="frame empty-frame"></td>
                        </tr>
                    })
                    .collect_view()
                }
            </table>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (_, rng) = signal_local(rand::rng());

    let dopts = Store::new(DataOptions::default());
    let vopts = Store::new(VisualOptions::default());

    let recorder = Recorder::new();

    let (values_r, values_w) = signal(gen_values(
            &mut rng.write_untracked(),
            dopts.generation().get_untracked(),
            dopts.shape().get_untracked(),
            dopts.size().get_untracked()
    ));

    // Need to choose the first, because the <select> element apparently chooses the first option
    // as well(??)
    let (algo_r, algo_w) = signal(ALGORITHMS[0]);

    let (history_r, history_w) = signal(vec![Vew::from(&*values_r.read_untracked())]);
    let (sorter_r, sorter_w) = signal_local(Coro::new(
            algo_r.get_untracked().func()(
                values_r.get_untracked(),
                recorder,
            )
    ));

    // When data options changes, reset values and sorter
    Effect::new(move |_| {
        values_w.set(gen_values(
                &mut rng.write_untracked(),
                dopts.generation().get(),
                dopts.shape().get(),
                dopts.size().get()
        ));

        history_w.write().clear();
        recorder.reset();
        sorter_w.set(Coro::new(
                algo_r.get_untracked().func()(
                    values_r.get(),
                    recorder,
                )
        ));

        if history_r.read_untracked().is_empty() {
            run_once(sorter_w, history_w);
        }
    });

    // When algorithm changes, just reset sorter
    Effect::new(move |_| {
        if history_r.read_untracked().is_empty() {
            sorter_w.set(Coro::new(
                    algo_r.get().func()(
                        values_r.get_untracked(),
                        recorder,
                    )
            ));

            run_once(sorter_w, history_w);
        } else {
            let last = Box::from(
                history_r.read_untracked()
                    .last()
                    .unwrap()
                    .list()
            );
            sorter_w.set(Coro::new(
                    algo_r.get().func()(last, recorder)
            ));
        }
    });

    view! {
        <main id="wasm">
            <div id="left">
                <Control
                    rng=rng
                    algo_r=algo_r
                    algo_w=algo_w
                    history_r=history_r
                    history_w=history_w
                    sorter_w=sorter_w
                    dopts=dopts
                    vopts=vopts
                    values_w=values_w
                    recorder=recorder
                />
            </div>
            <div id="content">
                <Content
                    dopts=dopts
                    vopts=vopts
                    history_r=history_r
                />
            </div>
            <div id="right">
                <Right
                    recorder=recorder
                />
            </div>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let handle = mount_to(
        document()
            .get_element_by_id("mountpoint")
            .unwrap()
            .unchecked_into(),
        App,
    );
    handle.forget();
}
