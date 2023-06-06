# Traitify

[![crates.io](https://img.shields.io/crates/v/traitify.svg)](https://crates.io/crates/traitify) [![Documentation](https://docs.rs/traitify/badge.svg)](https://docs.rs/traitify)

## Should you use this crate?

No. Probably not.

## Why did I make this crate?

Well, sometimes you have a type with generics and need to pass it to a function that cannot have generics.
It would be awesome if Rust had syntax to the likes of this:

```rust,ignore
struct Foo<BAR, const FIZZ: usize>;

fn buzz(foo: &Foo<dyn, dyn>) {}
```
See what I did there? This imaginary syntax of `dyn` in the place of a generic type would make this into a trait object on the fly! Cool right?

Sadly this syntax isn't real and there really isn't anything like it.

## So what are some alternatives that do work?

There are basically two thing I can come up with.

If you know all the different variants of your generics, you could create an enum and reimplement all `Foo` functions on that enum.

If you don't, then you could create a trait with all the functions of `Foo` and implement that trait for `Foo`. But writing a trait that is only implemented by one type feels wrong. It feels like writing a C header file and I'm done with writing headers.

That's where this crate comes in!

This crate only contains a macro you can apply to an `impl` block.
You tell it what the name of the trait needs to be and which generic types it should ignore.

```rust
use traitify::traitify;

struct Foo<BAR, const FIZZ: usize> {
    data: [BAR; FIZZ],
};

#[traitify(FooDynFizz, dyn = [FIZZ])]
#[traitify(FooDynBAR, dyn = [BAR])]
#[traitify(FooDynAll, dyn = [FIZZ, BAR])]
impl<BAR, const FIZZ: usize> Foo<BAR, FIZZ> {
    pub fn print_greeting(&self) {
        println!("Hallo vriendjes en vriendinnetjes!");
    }
}

#[test]
fn compiles() {
    let foo = Foo::<u8, 4> { data: [0; 4] };
    let dyn_foo: &dyn FooDynAll = &foo;
    dyn_foo.print_greeting();
}
```

Now, I tried my best to make this macro handle as much syntax as possible. But if you find something that doesn't work that should, then please file an issue or PR!

Here's a (possibly incomplete) list of rules of which functions end up in the generated trait:

- Function must be public (this is not a tech requirement, but otherwise you'd leak private functions through public traits)
- Function must not have a parameter of a generic type that is in the dyn-list.
- Function must not have an ABI specified
- Function must have a receiver parameter (self, &self or &mut self). It will be in the trait, but with a `Self: Sized` bound for object safety. Not terribly useful for trait objects, but it was easy to add.

You can't erase a lifetime. I think it should be possible, but I simply haven't done it yet.
