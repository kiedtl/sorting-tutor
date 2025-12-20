use leptos::prelude::*;
use leptos::html;

use crate::for_coro;
use crate::sorter::List;
use crate::recorder::Recorder;
use crate::utils::{
    Coro, IsSnapshot, Snapshot,
    list_into_view,
};

use std::pin::Pin;
use std::ops::{CoroutineState, Coroutine};

pub fn sort(mut x: List, r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        yield crate::utils::VList::new(&x).into();
        for_coro!(y in _sort(&mut x, r) =>
            yield y.build().into()
        );
    }
}

pub fn _sort(x: &mut [usize], r: Recorder) -> impl Coroutine<(), Yield = VMergeBuilder, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("mergesort");

        if x.len() < 2 {
            return;
        }

        let mid = x.len() / 2;
        let mut s1 = Box::from(&x[..mid]);
        let mut s2 = Box::from(&x[mid..]);

        for_coro!(y in _sort(&mut s1, r) =>
            yield y.layer(&x, 0)
        );

        for_coro!(y in _sort(&mut s2, r) =>
            yield y.layer(&x, mid)
        );

        for_coro!(y in merge(&s1, &s2, x, r) =>
            yield VMergeBuilder::new(y)
        );
    }
}

pub fn merge(
    s1: &[usize],
    s2: &[usize],
    x: &mut [usize],
    r: Recorder,
) -> impl Coroutine<(), Yield = VMergeSituation, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("merge");
        yield VMergeSituation::new(s1, s2, x, 0, 0, None);

        let (mut i, mut j) = (0, 0);
        while i + j < x.len() {
            if j == s2.len() || (i < s1.len() && r.lt(s1[i], s2[j])) {
                x[i + j] = s1[i];
                i += 1;
            } else {
                x[i + j] = s2[j];
                j += 1;
            }

            r.record_merge();
            yield VMergeSituation::new(
                s1, s2, x, i, j, Some(i + j - 1),
            );
        }
    }
}

#[derive(Clone)]
enum VMergeLayerKind {
    Top(VMergeSituation),
    Sub(Box<[usize]>),
}

#[derive(Clone)]
struct VMergeLayer {
    kind: VMergeLayerKind,
    start: usize,
}

#[derive(Clone)]
pub struct VMergeSituation {
    s1: Box<[usize]>,
    s2: Box<[usize]>,
    x: Box<[usize]>,
    s1ptr: usize,
    s2ptr: usize,
    added: Option<usize>,
}

impl VMergeSituation {
    pub fn new(
        s1: &[usize],
        s2: &[usize],
        x: &[usize],
        s1ptr: usize,
        s2ptr: usize,
        added: Option<usize>,
    ) -> Self
    {
        Self {
            s1: Box::from(s1),
            s2: Box::from(s2),
            x: Box::from(x),
            s1ptr, s2ptr, added,
        }
    }
}

pub struct VMergeBuilder {
    layers: Vec<VMergeLayer>,
}

impl VMergeBuilder {
    pub fn new(situation: VMergeSituation) -> Self {
        Self {
            layers: vec![
                VMergeLayer {
                    kind: VMergeLayerKind::Top(situation),
                    start: 0,
                }
            ],
        }
    }

    pub fn layer(mut self, parent: &[usize], child_start: usize) -> Self {
        for layer in &mut self.layers {
            layer.start += child_start;
        }
        self.layers.push(VMergeLayer {
            kind: VMergeLayerKind::Sub(parent.into()),
            start: 0,
        });

        self
    }

    pub fn build(self) -> VMerge {
        let base_list = match self.layers.last() {
            Some(VMergeLayer { kind: VMergeLayerKind::Sub(lst), .. }) => lst.clone(),
            Some(VMergeLayer { kind: VMergeLayerKind::Top(sit), .. }) => sit.x.clone(),
            None => unreachable!(),
        };
        let full_list_length = base_list.len();

        let mut current_state = Vec::from(base_list);
        for layer in self.layers.iter().rev().skip(1) {
            let list = match &layer.kind {
                VMergeLayerKind::Sub(lst) => lst,
                VMergeLayerKind::Top(VMergeSituation { x, .. }) => x,
            };
            for i in 0..list.len() {
                current_state[layer.start + i] = list[i];
            }
        }

        VMerge {
            layers: self.layers,
            full_list_length,
            current_state: current_state.into_boxed_slice(),
        }
    }

    // pub fn set_fleeting(mut self, value: bool) -> Self {
    //     self.fleeting = value;
    //     self
    // }
}

pub struct VMerge {
    layers: Vec<VMergeLayer>,
    full_list_length: usize,
    current_state: Box<[usize]>,
}

impl IsSnapshot for VMerge {
    fn swapped(&self) -> Option<(usize, usize)> {
        None
    }

    fn list(&self) -> &[usize] {
        &self.current_state
    }

    fn is_fleeting(&self) -> bool {
        match self.layers.last() {
            Some(VMergeLayer {
                kind: VMergeLayerKind::Top(situation),
                ..
            }) => situation.added.is_none(),
            _ => false, // ??
        }
    }

    fn into_view(&self) -> AnyView {
        let full_list_length = self.full_list_length;
        let layers = self.layers.clone();
        let current_state = self.current_state.clone();

        view! {
            <div class="group">
                // <div class="enclosure">
                //     { list_into_view(0, 0, &current_state, None, None, None)}
                // </div>
                {
                    layers.iter().map(|layer| {
                        match &layer.kind {
                            VMergeLayerKind::Top(situation) => {
                                // let until = situation.s1ptr + situation.s2ptr;
                                let end_pad = full_list_length - layer.start - situation.x.len(); //(layer.start + situation.x.len() - until);
                                view! {
                                    <div class="enclosure">
                                        {list_into_view(
                                            layer.start,
                                            end_pad, // + situation.x.len().saturating_sub(until),
                                            &situation.x, //[..until.min(situation.x.len())],
                                            None,
                                            situation.added,
                                            None
                                        ).into_any()}
                                    </div>
                                    <div class="solo-group mergesort-child-group">
                                        <div class="enclosure">
                                            {list_into_view(0, 0, &situation.s1, None, None, Some(situation.s1ptr))}
                                        </div>
                                        <div class="enclosure">
                                            {list_into_view(0, 0, &situation.s2, None, None, Some(situation.s2ptr))}
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            VMergeLayerKind::Sub(values) => {
                                let end_pad = full_list_length - values.len() - layer.start;
                                html::div()
                                    .class("enclosure")
                                    .child(
                                        list_into_view(layer.start, end_pad, &values, None, None, None)
                                    )
                                    .into_any()
                            },
                        }
                    })
                    .collect_view()
                }
            </div>
        }.into_any()
    }
}
