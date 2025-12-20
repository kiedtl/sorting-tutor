+++
title = "About"
+++

## About

Sorting Tutor is a [WebAssembly](https://en.wikipedia.org/wiki/WebAssembly) app,
built in [Rust](https://rust-lang.org/) using the [Leptos](https://leptos.dev/)
framework. Static content is made with [Zola](https://getzola.org). The source
is available on
[GitHub](https://github.com/kiedtl/sorting-tutor) (under the MIT licence for the
code, and CC-BY-NC-ND for static content).

<p>
<a target="_blank" href='https://github.com/kiedtl/sorting-tutor'>
	<img class="inline badge" src='//tilde.team/~kiedtl/images/badges/source/github-dark.png' />
</a>

<a target="_blank" href='https://512kb.club/#512'>
	<img class="inline badge" src='//tilde.team/~kiedtl/images/badges/club/512kb-club.png' />
</a>

<a target="_blank" href='https://github.com/kiedtl/badges'>
	<img class="inline badge" src='//tilde.team/~kiedtl/images/badges/rust/cultist-dark-rust.png' />
</a>

<a target="_blank" href='https://github.com/kiedtl/badges'>
	<img class="inline badge" src='//tilde.team/~kiedtl/images/badges/rust/lifetime-dark.png' />
</a>
</p>

This site is a work in progress. All comments, suggestions, bikeshedding, and related hatemail (`kiedtl at <current website> dot team`) are appreciated.

### Prior Art

Sorting Tutor takes inspiration from two other algorithm visualizers:

- [The Sound of Sorting](https://mszula.github.io/visual-sorting/)
- [VisuAlgo](https://visualgo.net/en/sorting)

I'm aware there are around four billion more, but the above were the main
inspirations. However, two key differentiators hopefully set this website apart:
1. A more pedagogical focus, to help one grok the details of how an algorithm
   works (in addition to getting an intuitive understanding of how elements are
   shuffled around and reordered). Thus the focus on showing the internal state
   of the sorting algorithm, along with short one-line explainers.
2. Access to performance metrics.

{% sidenote() %}
If you're using this site in a course you teach, I'd be happy to know — that's
what this project was for in the first place, and knowing that it's in use can
be encouraging. Feature requests are welcome as well.
{% end %}

### Roadmap

A rough list of new features that will eventually be added (*contributions
welcome*).

1. Common/basic sorting algorithms.
   - Mergesort
   - Shellsort
   - Counting, bucket, radix sort
3. Ability to set algorithm-specific options.
   - ex. Pivot heuristic for Quicksort (Lomuto, median-of-three, median-of-nine,
     random)
4. More sorting algorithms, especially ones used in the standard libraries of
   various programming languages.
   - Timsort
   - Powersort
   - Driftsort
2. Metrics on memory usage, i.e. space complexity.
3. More kinds of visualization.

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
need to manually craft a state machine — one simply writes the sorting algorithm
as normal, and sprinkles in `yield` statements where a snapshot of the
partially-sorted data is desired.

```
pub fn selection_sort(mut x: List) -> impl Coroutine<(), Yield = Snapshot, Return = ()> {
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
Don't be frightened by the `-> impl Coroutine<...>` — it simply indicates that
the function returns something that implements the `Coroutine`
interface, with the following properties:

- It takes void (`()`, the empty tuple) as an argument,
- It yields a `Snapshot`, and
- It returns void.

The `Coroutine` interface defines the `resume()` method, among others.

Internally, the Rust compiler takes our coroutine and does the hard work of
desugaring it into a state machine that implements this interface — similar to
how `async` functions are desugared into `Future`s.
{% end %}
