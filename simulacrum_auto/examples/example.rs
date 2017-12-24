#![feature(proc_macro)]

extern crate simulacrum_auto;
extern crate simulacrum_macros;

use simulacrum_macros::*;
use simulacrum_auto::simulacrum;

#[simulacrum]
trait CoolTrait {
    // Shared self
    fn foo(&self);

    // Mutable self
    fn bar(&mut self);

    // One parameter and returning a value
    fn goop(&mut self, flag: bool) -> u32;

    // Multiple parameters
    fn zing(&self, first: i32, second: bool);

    // Static reference
    fn boop(&self, name: &'static str);

    // Shared reference
    fn store(&self, val: &i64);

    // Mutable reference
    fn toggle(&self, bit: &mut bool);
}

fn main() {
    // Set up expectations
    let mut m = CoolTraitMock::new();
    m.expect_bar().called_never();
    m.expect_foo().called_once();
    m.then().expect_goop().called_once().with(true).returning(|_| 5);
    m.then().expect_zing().called_once().with(params!(13, false));
    m.expect_boop().called_times(2);
    m.expect_store().called_once().with(deref(777));
    m.expect_toggle().called_once().with(deref(true))
                                   .returning(|&arg| { unsafe { *arg.as_mut().unwrap() = false } });

    // Execute test code
    m.foo();
    assert_eq!(m.goop(true), 5);
    m.zing(13, false);
    m.boop("hey");
    m.boop("yo");
    m.store(&777);
    let mut b = true;
    m.toggle(&mut b);
    assert_eq!(b, false);

    // When the Expectations struct is dropped, each of its expectations will be evaluated
}