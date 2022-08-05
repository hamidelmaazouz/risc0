use guestio_derive::{Serialize, Deserialize};
use crate as guestio;
use guestio::{serialize, deserialize::Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Foo {
    firstval: u32,
    b: std::vec::Vec<u32>
}

#[test]
pub fn test_foo() {
    let f=Foo {
        firstval:5, b:vec![1, 2, 3]
    };

    println!("Orig: {:?}", f);
    let be = serialize(&f).unwrap();

    println!("Serialized: {:?}", be);

    let de = Foo::deserialize_from(&be);

    let mys : Foo = de.into();

    eprintln!("Deserialized: {:?}", mys);

    assert_eq!(f, mys);
}
