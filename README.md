# `::nougat` <a href="https://www.flaticon.com/free-icon/nougat_2255580"><img src="https://user-images.githubusercontent.com/9920355/170709986-aaa13f92-0faf-4b5d-89c9-6463b6b3d49b.png" title="nougat logo from https://www.flaticon.com/free-icon/nougat_2255580" alt="nougat logo" width="25" /></a>

Use (lifetime-)GATs on stable rust.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/nougat.rs)
[![Latest version](https://img.shields.io/crates/v/nougat.svg)](
https://crates.io/crates/nougat)
[![Documentation](https://docs.rs/nougat/badge.svg)](
https://docs.rs/nougat)
[![MSRV](https://img.shields.io/badge/MSRV-1.60.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/nougat.svg)](
https://github.com/danielhenrymantilla/nougat.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/nougat.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/nougat.rs/actions)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

## Example

```rust
#![forbid(unsafe_code)]

#[macro_use]
extern crate nougat;

#[gat]
trait LendingIterator {
    type Item<'next>
    where
        Self : 'next,
    ;

    fn next(&mut self)
      -> Option<Self::Item<'_>>
    ;
}

struct WindowsMut<Slice, const SIZE: usize> {
    slice: Slice,
    start: usize,
}

#[gat]
impl<'iter, Item, const SIZE: usize>
    LendingIterator
for
    WindowsMut<&'iter mut [Item], SIZE>
{
    type Item<'next>
    where
        Self : 'next,
    =
        &'next mut [Item; SIZE]
    ;

    /// For reference, the signature of `.array_chunks_mut::<SIZE>()`'s
    /// implementation of `Iterator::next()` would be:
    /** ```rust ,ignore
    fn next<'next> (
        self: &'next mut AChunksMut<&'iter mut [Item], SIZE>,
    ) -> Option<&'iter mut [Item; SIZE]> // <- no `'next` nor "lending-ness"! ``` */
    fn next<'next> (
        self: &'next mut WindowsMut<&'iter mut [Item], SIZE>,
    ) -> Option<&'next mut [Item; SIZE]> // <- `'next` instead of `'iter`: lending!
    {
        let to_yield =
            self.slice
                .get_mut(self.start ..)?
                .get_mut(.. SIZE)?
                .try_into() // `&mut [Item]` -> `&mut [Item; SIZE]`
                .expect("slice has the right SIZE")
        ;
        self.start += 1;
        Some(to_yield)
    }
}

fn main() {
    let mut array = [0, 1, 2, 3, 4];
    let slice = &mut array[..];
    // Cumulative sums pattern:
    let mut windows_iter = WindowsMut::<_, 2> { slice, start: 0 };
    while let Some(item) = windows_iter.next() {
        let [fst, ref mut snd] = *item;
        *snd += fst;
    }
    assert_eq!(
        array,
        [0, 1, 3, 6, 10],
    );
}
```

## Debugging / tracing the macro expansions

You can make the macros go through intermediary generated files so as to get
well-spanned error messages and files which you can open and inspect yourself,
with the remaining macro non-expanded for readability, by:

 1. enabling the `debug-macros` Cargo feature of this dependency:

    ```toml
    [dependencies]
    ## …
    nougat.version = "…"
    nougat.features = ["debug-macros"]  # <- ADD THIS
    ```

 1. Setting the `DEBUG_MACROS_LOCATION` env var to some _absolute_ path where
    the macros will write the so-generated files.

### Demo

[<img src="https://i.imgur.com/0yyQVJf.gif" height="250" alt="demo"/>](
https://i.imgur.com/0yyQVJf.gif)

## How does the macro work?

<detais><summary>Click here to see an explanation of the implementation</summary>

#### Some historical context

 1. **2021/02/24**: [Experimentation with `for<'lt> Trait<'lt>` as a super-trait
    to emulate GATs](https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/What.20will.20GATs.20allow.20streaming.20iterators.20to.20do.20differently.3F/near/228154288)

      - (I suspect there may even be previous experimentations and usages over
        URLO; but I just can't find them at the moment)

    This already got GATs almost done, but for two things, regarding which I did
    complain at the time 😅:

      - The `Trait<'lt>` embedded _all_ the associated items, including the
        methods, and not just the associated "generic" type.

        This, in turn, could lead to problems if these other items relied on
        the associated type being _fully generic_, as I observe [here](
        https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/What.20will.20GATs.20allow.20streaming.20iterators.20to.20do.20differently.3F/near/229123071), on the **2021/03/06**.

      - I was unable to express the `where Self : 'next` GAT-bounds.

 1. **2021/03/08**: [I officially mention the workaround for
    "_late_/`for`-quantifying `where T : 'lt`" clauses thanks implicit bounds
    on types such as `&'lt T`](https://users.rust-lang.org/t/how-to-end-borrow-in-this-code/72719/2?u=yandros).

      - For those interested, I used this technique, later on, to work around
        a nasty "overly restrictive lifetime-bound in higher-order closure
        context" issue in [a very detailed URLO post that you may thus find
        interesting](https://users.rust-lang.org/t/argument-requires-that-is-borrowed-for-static/66503/2?u=yandros).


</details>

### Limitations

  - Only _lifetime_ GATs are supported (no `type Assoc<T>` nor
    `type Assoc<const …>`).

  - The code generated by the macro is currently **not `dyn`-friendly** _at all_.
    This will likely be improved in the future; potentially using another
    desugaring for the implementation.

  - In order to refer to GATs outside of
    <code>[#\[gat\]]</code>-annotated items using [`Gat!`] is needed.

[`Gat!`]: https://docs.rs/nougat/0.1.*/nougat/macro.Gat.html
[#\[gat\]]: https://docs.rs/nougat/0.1.*/nougat/att.gat.html
