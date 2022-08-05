#![cfg(test)]

use super::*;
use guestio_derive::{Serialize, Deserialize};
use crate as guestio;

#[derive(Debug, Serialize, Deserialize)]
struct MyStruct {
    foo: u32,
    bar: std::string::String,
    next: Option<Box<MyStruct>>,
}

#[test]
pub fn ser_test() {
    let a = MyStruct {
        foo: 5,
        bar: "hi".into(),
        next: None,
    };
    println!("Orig: {:?}", a);

    let b = serialize(&a).unwrap();

    println!("Serialized: {:?}", b);

    let des = MyStruct::deserialize_from(&*b);

    println!("foo: {}", des.foo());
    println!("bar: {}", des.bar());

    let mys: MyStruct = des.into();
    println!("Deserialized: {:?}", mys);
}

pub fn memory_barrier<T>(ptr: *const T) {
    use core::arch::asm;
    // SAFETY: This passes a pointer in, but does nothing with it.
    unsafe { asm!("/* {0} */", in(reg) (ptr)) }
}

#[test]
pub fn ser_test_link() {
    let a = MyStruct {
        foo: 5,
        bar: "hi".into(),
        next: Some(Box::new(MyStruct {
            foo: 6,
            bar: "Why hello there!".into(),
            next: None,
        })),
    };
    println!("Orig: {:?}", a);

    let b = serialize(&a).unwrap();

    println!("Serialized: {:?}", b);

    let des = MyStruct::deserialize_from(&*b);
    println!("foo: {}", des.foo());
    println!("bar: {}", des.bar());

    let mys: MyStruct = des.into();
    println!("Deserialized: {:?}", mys);
}
