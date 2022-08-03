use criterion::{criterion_group, criterion_main, Criterion};

use core::convert::TryFrom;
use risc0_zkvm::{method_id::MethodId, receipt::Receipt};
use std::vec::Vec;

const RECEIPT_DATA: &[u8] = include_bytes!("simple_receipt.receipt");
const METHOD_ID: &[u8] = include_bytes!("simple_receipt.id");

fn receipt(c: &mut Criterion) {
    let receipt_data: Vec<u32> = RECEIPT_DATA
        .chunks(4)
        .map(|bytes| u32::from_le_bytes(<[u8; 4]>::try_from(bytes).unwrap()))
        .collect();

    let receipt: Receipt = risc0_zkvm::serde::from_slice(&receipt_data).unwrap();

    let method_id = MethodId::from_slice(METHOD_ID).unwrap();
    c.bench_function("verify", move |b| {
        b.iter(|| receipt.verify(&method_id));
    });
}

criterion_group!(benches, receipt);
criterion_main!(benches);
