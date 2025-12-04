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

use gloo::timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_use::*;
use rand::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (::web_sys::console::log_1(&format!($($t)*).into()))
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

#[component]
fn Control(
    algo_r: ReadSignal<Algorithm>,
    algo_w: WriteSignal<Algorithm>,
    history_r: ReadSignal<Vec<Vew>>,
    history_w: WriteSignal<Vec<Vew>>,
    sorter_w: WriteSignal<Coro<Vew>, LocalStorage>,
    size_r: ReadSignal<usize>,
    size_w: WriteSignal<usize>,
    recorder: Recorder,
) -> impl IntoView
{
    let (running_r, running_w) = signal(false);
    let (delay_r, delay_w) = signal(60);

    view! {
        <h3>"Control"</h3>
        <table>
            <tr>
                <td>
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
                </td>
                <td>
                    <button
                        on:click=move |_| run_once(sorter_w, history_w)
                    >
                    "Step"
                    </button>
                </td>
            </tr>
        </table>
        <h3>"Settings"</h3>
        <table>
            <tr>
                <td>
                    <label>Algorithm</label>
                </td>
                <td>
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
                    <label for="Size">Size</label>
                </td>
                <td>
                    <input
                        type="range" id="size" name="Size" min="4" max="96"
                        value=move || size_r.get()
                        on:input:target=move |ev| {
                            size_w.set(ev.target().value().parse().unwrap());
                        }
                    />
                </td>
                <td>
                    <i>{move || size_r.get()}</i>
                </td>
            </tr>
            <tr>
                <td>
                    <label for="Delay">Delay</label>
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
                <td>
                    <i>{move || delay_r.get()}"ms"</i>
                </td>
            </tr>
        </table>
        <h3>"Stats"</h3>
        <table>
            <tr>
                <td><label>"Comparisons"</label></td>
                <td>{move || recorder.count_comparisons()}</td>
            </tr>
            <tr>
                <td><label>"Swaps"</label></td>
                <td>{move || recorder.count_swaps()}</td>
            </tr>
        </table>
        <h3>"Legend"</h3>
        <table>
            <tr>
                <td class="head"><span style="width:auto" class="elem">"White"</span></td>
                <td>"Ordinary elements."</td>
            </tr>
            <tr>
                <td><label><span style="width:auto" class="elem swp">"Blue"</span></label></td>
                <td>"Elements just swapped."</td>
            </tr>
            <tr>
                <td><label><span style="width:auto" class="elem spc">"Gold"</span></label></td>
                <td>"\"Special\" elements (pivots, minimums)."</td>
            </tr>
        </table>
    }
}

#[component]
fn Content(
    history_r: ReadSignal<Vec<Vew>>,
) -> impl IntoView
{
    // let bars = move || history_r.read().last().map(|v| v.list().len()).unwrap_or(0);
    // let bar_width_fac = move || if bars() > 32 { 1. } else { 1.7 };
    // let bar_width = move || bars() as f32 * bar_width_fac();

    let bars = move || history_r.read().last().unwrap().list().iter().copied().enumerate().collect::<Vec<_>>();
    let bars_len = move || history_r.read().last().unwrap().list().len();
    let were_bars_swapped = move |i| match history_r.read().last().unwrap().swapped() {
        Some((a, b)) if i == a || i == b => true,
        _ => false,
    };

    view! {
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
                        style=move || format!("height:{v}px")
                    >
                    </div>
                </For>
            </div>
        </div>
        <For
            each=move || {
                let mut found_important = false;
                history_r
                    .read()
                    .iter()
                    .enumerate()
                    .rev()
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
            key=|&(k, h)| (k, h)
            let((k, _))
        >
        {move || history_r.read()[k].into_view()}
        </For>
    }
}

#[component]
fn App() -> impl IntoView {
    let mut rng = rand::rng();

    // Default size is power of two minus two -- good for heapsort, since it
    // means an "almost-full" tree
    let (size_r, size_w) = signal(14);

    let recorder = Recorder::new();

    let (values_r, values_w) = signal({
        (0..size_r.get_untracked())
            .map(|_| rng.random_range(10..99))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });

    // Need to choose the first, because the <select> element apparently chooses the first option
    // as well(??)
    let (algo_r, algo_w) = signal(ALGORITHMS[0]);

    let (history_r, history_w) = signal(vec![Vew::from(&*values_r.read_untracked())]);
    let (_, sorter_w) = signal_local(Coro::new(
            algo_r.get_untracked().func()(
                values_r.get_untracked(),
                recorder,
            )
    ));

    // When size/values/algorithm changes, set values
    Effect::new(move |_| {
        if size_r.get() != values_r.read().len() {
            values_w.set({
                (0..size_r.get())
                    .map(|_| rng.random_range(10..99))
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            });
        }

        history_w.write().clear();
        recorder.reset();
        sorter_w.set(Coro::new(
                algo_r.get().func()(
                    values_r.get_untracked(),
                    recorder,
                )
        ));
        run_once(sorter_w, history_w);
    });

    view! {
        <table>
            <tr>
                <td style="vertical-align: top">
                    <Control
                        algo_r=algo_r
                        algo_w=algo_w
                        history_r=history_r
                        history_w=history_w
                        sorter_w=sorter_w
                        size_w=size_w
                        size_r=size_r
                        recorder=recorder
                    />
                </td>
                <td>
                    <Content
                        history_r=history_r
                    />
                </td>
            </tr>
        </table>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {
        <main>
            <App/>
        </main>
    });
}
