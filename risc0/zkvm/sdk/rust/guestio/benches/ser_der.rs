use core::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guestio::{deserialize::Deserialize, serialize};
use guestio_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct MyStruct {
    foo: u32,
    bar: std::string::String,
    next: Option<Box<MyStruct>>,
}

fn make_struct() -> MyStruct {
    MyStruct {
        foo: 5,
        bar: "hi".into(),
        next: Some(Box::new(MyStruct {
            foo: 6,
            bar: "Why hello there!".into(),
            next: None,
        })),
    }
}

pub fn bench(c: &mut Criterion) {
    let a = make_struct();
    let serd = serialize(&a).unwrap();
    if false {
        c.bench_function("ser_der",  |b| {
            b.iter(|| {
                let b = serialize(&a).unwrap();
                let des = MyStruct::deserialize_from(&*b);
                black_box(des);
            })
        });
    }
    if false {
        c.bench_function("ser",  |b| {
            b.iter(|| {
                let b = serialize(&a).unwrap();
                black_box(b);
            })
        });
    }
    if true {
        c.bench_function("der",  |b| {
            b.iter(|| {
                let des = MyStruct::deserialize_from(black_box(&*serd));
                black_box(&des);
                black_box(des.next().unwrap().foo());
            })
        });
    }
}

criterion_group!(name = benches;
                 config = Criterion::default().measurement_time(Duration::new(30,0));
                                  targets = bench);
criterion_main!(benches);
