use leptos::prelude::*;
use leptos::html;

use crate::for_coro;
use crate::sorter::List;
use crate::recorder::Recorder;
use crate::utils::{ Coro, IsSnapshot, Snapshot, VList };

use std::pin::Pin;
use std::ops::{Range, CoroutineState, Coroutine};

mod gap_sequences {
    pub const CIAURA: &[usize] = &[701, 301, 132, 57, 23, 10, 4, 1];
}

pub fn sort(mut x: List, mut r: Recorder) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
    #[coroutine] static move || {
        let _g = r.f("shell");
        yield VList::new(&x).into();
        let n = x.len();

        for &gap in gap_sequences::CIAURA {
            // Not strictly needed since if gap >= n nothing happens anyway, but nice to not yield
            // a VList Snapshot for nothing.
            if gap >= n {
                continue;
            }

            yield VList::new(&x)
                .expl(format!("Gap: {gap}"))
                .into();

            for i in gap..n {
                let tmp = x[i];
                let mut j = i;

                yield VList::new(&x)
                    .expl(format!("i = {i}"))
                    .special(i)
                    .into_snapshot()
                    .fleeting();

                while j >= gap && r.gt(x[j - gap], tmp) {
                    r.swap(&mut x, j, j - gap);
                    yield VList::new(&x)
                        .current(j)
                        .expl(format!("Comparing with {tmp}"))
                        .swapped(j, j - gap)
                        .into();
                    j -= gap;
                }

                let nothing_done = x[j] == tmp;
                x[j] = tmp;
                r.record_swap();

                if nothing_done {
                    yield VList::new(&x)
                        .expl("Did nothing")
                        .into_snapshot()
                        .fleeting();
                } else {
                    yield VList::new(&x)
                        .swapped(j, j)
                        .expl("Restored value")
                        .current(j)
                        .into();
                }
            }
        }
    }
}
