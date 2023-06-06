use std::{fmt::Debug, ops::AddAssign};
use traitify::traitify;

#[derive(Debug)]
struct Foo<T, const CAP: usize> {
    data: [T; CAP],
}

#[traitify(FooDynCap, dyn = [CAP])]
#[traitify(FooDynT, dyn = [T])]
#[traitify(FooDynAll, dyn = [CAP, T])]
impl<T: Default + Copy + Debug, const CAP: usize> Foo<T, CAP>
where
    T: AddAssign<u8>,
    T: AddAssign<T>,
{
    pub fn new() -> Self {
        Self {
            data: [T::default(); CAP],
        }
    }

    fn print_dbg(&self) {
        dbg!(self);
    }

    pub fn add_one(&mut self) {
        for i in self.data.iter_mut() {
            *i += 1;
        }
    }

    pub fn add_n(&mut self, n: T) {
        for i in self.data.iter_mut() {
            *i += n;
        }
    }

    pub fn print_hello() {
        println!("Hello!");
    }
}

#[test]
fn compiles() {
    let mut foo = Foo::<u8, 4>::new();
    let dyn_foo: &mut dyn FooDynCap<u8> = &mut foo;
    dyn_foo.add_one();
    dyn_foo.add_n(5);
    foo.print_dbg();

    let dyn_foo: &mut dyn FooDynT<4> = &mut foo;
    dyn_foo.add_one();
    foo.print_dbg();

    let dyn_foo: &mut dyn FooDynAll = &mut foo;
    dyn_foo.add_one();
    foo.print_dbg();
}
