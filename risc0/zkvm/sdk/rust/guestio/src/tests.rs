use super::{serialize, Deserialize, Serialize, PAD_WORDS};
use crate as guestio;
use core::fmt::Debug;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
struct StructContainer<T: Eq + Serialize + Debug + for<'a> Deserialize<'a>> {
    a: u32,
    b: T,
    c: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct StructContainer2<T: Debug + Serialize + Eq + Clone + for<'a> Deserialize<'a>> {
    a: u32,
    b: Option<T>,
    c: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct StructContainer3<T: Debug + Serialize + Eq + Clone + for<'a> Deserialize<'a>> {
    a: u32,
    b: Box<StructContainer2<T>>,
    c: u32,
}

pub fn test_round_trip_no_containers<
    T: Serialize + for<'a> Deserialize<'a> + Eq + Debug + Clone,
>(
    val: &T,
) {
    let u32s: Vec<u32> = serialize(val).unwrap();
    assert_eq!(u32s.len() % PAD_WORDS, 0);

    let deser = T::deserialize_from(&u32s);
    let actual = T::into_orig(deser);

    assert_eq!(*val, actual);
}

pub fn test_round_trip<T: Serialize + for<'a> Deserialize<'a> + Eq + Debug + Clone>(val: &T) {
    test_round_trip_no_containers(val);

    test_round_trip_no_containers(&StructContainer::<T> {
        a: 1,
        b: val.clone(),
        c: 2,
    });

    test_round_trip_no_containers(&Vec::<StructContainer2<T>>::from([
        StructContainer2::<T> {
            a: 3,
            b: None,
            c: 4,
        },
        StructContainer2::<T> {
            a: 5,
            b: Some(val.clone()),
            c: 6,
        },
    ]));

    test_round_trip_no_containers(&Vec::<Option<StructContainer3<T>>>::from([
        Some(StructContainer3::<T> {
            a: 7,
            b: Box::from(StructContainer2::<T> {
                a: 8,
                b: Some(val.clone()),
                c: 9,
            }),
            c: 10,
        }),
        None,
    ]));
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
struct SimpleStruct {
    a: u32,
}

#[test]
fn basic_types() {
    test_round_trip(&0u32);
    test_round_trip(&1u32);
    test_round_trip(&u32::MAX);
    test_round_trip(&SimpleStruct { a: 3 });
    test_round_trip(&String::from(""));
    test_round_trip(&String::from("How"));
    test_round_trip(&String::from("Howd"));
    test_round_trip(&String::from("Howdy"));
    test_round_trip(&[1u32, 2, 3]);
    test_round_trip(&[1u32, 2, 3, 4]);
    test_round_trip(&[1u32, 2, 3, 4, 5]);
}
