use std::{fmt::Debug, ops::AddAssign};
use traitify::traitify;

#[derive(Debug)]
struct Foo<T, const CAP: usize> {
    data: [T; CAP],
}

#[traitify(FooTrait, dyn = [CAP])]
impl<T: Default + Copy + Debug, const CAP: usize> Foo<T, CAP>
where
    T: AddAssign<u8>,
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
}

#[test]
fn compiles() {
    let mut foo = Foo::<u8, 4>::new();
    let dyn_foo: &mut dyn FooTrait<u8> = &mut foo;
    dyn_foo.add_one();
    foo.print_dbg();
}
