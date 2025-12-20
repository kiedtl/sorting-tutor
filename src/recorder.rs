use leptos::prelude::{Write, WriteSignal, Read, ReadSignal, Set, signal};

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

// Most of this is a wrapper for RecorderState. FIXME: Use Deref or something to do this
// automatically.
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

    pub fn record_merge(&self) {
        self.0.write().record_merge();
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
        self.comparisons += 1;
        a < b
    }

    pub fn gt(&mut self, a: usize, b: usize) -> bool {
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

    pub fn record_merge(&mut self) {
        self.swaps += 1;
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
