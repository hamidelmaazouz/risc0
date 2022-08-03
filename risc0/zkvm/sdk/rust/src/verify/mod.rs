// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(test)]
mod tests {
    use crate::method_id::MethodId;
    use crate::receipt::Receipt;
    use risc0_zkp::core::sha::DIGEST_WORD_SIZE;
    use std::{convert::TryFrom, vec::Vec};

    #[test]
    fn test_receipt() {
        const RECEIPT_DATA: &[u8] = include_bytes!("../../benches/simple_receipt.receipt");
        const METHOD_ID: &[u8] = include_bytes!("../../benches/simple_receipt.id");

        let receipt_data: Vec<u32> = RECEIPT_DATA
            .chunks(DIGEST_WORD_SIZE)
            .map(|bytes| u32::from_le_bytes(<[u8; DIGEST_WORD_SIZE]>::try_from(bytes).unwrap()))
            .collect();
        let receipt: Receipt = crate::serde::from_slice(&receipt_data).unwrap();

        let method_id = MethodId::from_slice(METHOD_ID).unwrap();

        std::println!(
            "Receipt: journal length {} seal length {}",
            receipt.journal.len(),
            receipt.seal.len(),
        );

        for i in 0..50 {
            std::print!(" {}", receipt.seal[i]);
        }
        std::println!("\n");

        receipt.verify(&method_id).unwrap();
    }
}
