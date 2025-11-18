#![allow(dead_code)]
#![allow(unused)]

#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(gen_blocks)]

mod sorter;

use crate::sorter::{Algorithm, ALGORITHMS, List};

use std::pin::Pin;
use std::ops::Coroutine;

use gloo::timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_use::*;
use rand::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
use web_sys::console;

macro_rules! log {
    ($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

#[component]
fn Control(
    algo_r: ReadSignal<Algorithm>,
    algo_w: WriteSignal<Algorithm>,
    history_r: ReadSignal<Vec<Box<[usize]>>>,
    history_w: WriteSignal<Vec<Box<[usize]>>>,
    sorter_w: WriteSignal<sorter::Sorter, LocalStorage>,
    size_r: ReadSignal<usize>,
    size_w: WriteSignal<usize>,
) -> impl IntoView
{
    let (running_r, running_w) = signal(false);
    let (delay_r, delay_w) = signal(20);

    view! {
        <table>
            <tr>
                <td>
                    <label>Algorithm</label>
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
                    <button
                        on:click=move |_| {
                            if running_r.get() {
                                running_w.set(false);
                            } else {
                                running_w.set(true);
                                spawn_local(async move {
                                    while let Some(view) = sorter_w.write().iter() && running_r.get() {
                                        history_w.write().push(view);
                                        TimeoutFuture::new(delay_r.get() as u32).await;
                                    }
                                    running_w.set(false);
                                });
                            }
                        }
                    >
                        {move || if running_r.get() { "Stop" } else { "Start" }}
                    </button>
                    <button
                        on:click=move |_| {
                            match sorter_w.write().iter() {
                                Some(view) => history_w.write().push(view),
                                None => (),
                            }
                        }
                    >
                    "Step"
                    </button>
                </td>
            </tr>
            <tr>
                <td>
                    <label for="Size">Size</label>
                </td>
                <td>
                    <input
                        type="range" id="size" name="Size" min="4" max="64"
                        value=move || size_r.get()
                        on:input:target=move |ev| {
                            size_w.set(ev.target().value().parse().unwrap());
                        }
                    />
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
            </tr>
        </table>
    }
}

#[component]
fn Content(
    history_r: ReadSignal<Vec<Box<[usize]>>>,
) -> impl IntoView
{
    view!{
        <table>
            <tr>
            {move || history_r.read().last().map(|values: &Box<[usize]>| {
                values.iter().copied().map(|v| view! {
                    <td style="vertical-align: bottom">
                        <div style=move || format!("background:green; height: {v}px")>
                        </div>
                    </td>
                }).collect_view()
            })}
            </tr>
        </table>
        <table>
            {move || history_r.read().iter().rev().map(|vset: &Box<[usize]>| {
                let vs = vset.iter().copied().map(|v| view! {
                    <td>{v}</td>
                }).collect_view();
                view! {
                    <tr>{vs}</tr>
                }
            }).collect_view()}
        </table>
    }
}

#[component]
fn App() -> impl IntoView {
    let mut rng = rand::rng();

    let (size_r, size_w) = signal(8);

    let (values_r, values_w) = signal({
        (0..size_r.get())
            .map(|_| rng.random_range(10..99))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });

    let (history_r, history_w) = signal(vec![values_r.get()]);
    let (algo_r, algo_w) = signal(Algorithm::Insertion);
    let (_, sorter_w) = signal_local(sorter::Sorter::new(algo_r.get().func()(values_r.get())));

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
        history_w.write().push(values_r.get());
        sorter_w.set(sorter::Sorter::new(algo_r.get().func()(values_r.get())));
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
