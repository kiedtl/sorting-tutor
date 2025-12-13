+++
title = "About"
+++

## About

Sorting Tutor is built in
[WebAssembly](https://en.wikipedia.org/wiki/WebAssembly), in
[Rust](https://rust-lang.org/) using the [Leptos](https://leptos.dev/)
framework. The source is available under the MIT license on
[GitHub](https://github.com/kiedtl/sorting-tutor).


### Architecture

This project uses the unstable `Coroutines` Rust feature — which, if you're not
familiar, allows writing resumable functions of the following form:

```
#[coroutine] move || {
    do_some_work();

    let value = do_some_more_work();
    yield value;

    for i in 0..10 {
        yield do_work_at_this_place(i);
    }

    return 37;
}
```

{% sidenote() %}
If you know Rust, note the `#[coroutine]` annotation that marks the closure.

If you're not familiar with Rust, then, well, this is basically a lambda that
captures (and *owns*, hence the `move` keyword) its environment. You'd typically
wrap this inside another function that returns the lambda.
{% end %}

This function can then be called repeatedly. A (heavily) simplified form would look like
this:

```
let coroutine = my_coroutine(); // Returns the resumable function as an object
while let CoroutineState::Yielded(value) = coroutine.resume() {
    println!("Coroutine yielded: {}", value);
}
```

Naturally, this makes writing the sorting functions much easier, as there is no
need to manually write a state machine — one simply writes the sorting algorithm
as normal, and sprinkles in `yield` statements where a snapshot of the
partially-sorted data is desired.

```
pub fn selection_sort(mut x: List) -> impl Coroutine<(), Yield = (), Return = ()> {
    #[coroutine] static move || {
        for i in 0..(x.len() - 1) {
            let min = (i..x.len())
                .min_by_key(|i| x[i])
                .unwrap();

            x.swap(i, min);
            yield create_snapshot_of_list(x);
        }
    }
}
```

{% sidenote() %}
If you don't know Rust, don't be frightened by the `-> impl Coroutine<...>` — it
simply indicates that the `selection_sort()` function returns a type that
implements the `Coroutine` interface, allows us to `resume()` it. Internally,
the Rust compiler takes our coroutine and desugars it into a state machine that
implements this interface.
{% end %}
