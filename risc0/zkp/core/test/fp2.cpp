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

#include "risc0/zkp/core/fp2.h"
#include "risc0/core/rng.h"

#include <gtest/gtest.h>

namespace risc0 {

TEST(Fp2, IsaField) {
  PsuedoRng rng(2);
  // Pick random sets of 3 elements of Fp2, and verify they meet the requirements of a field.
  for (size_t i = 0; i < 1000000; i++) {
    Fp2 a = Fp2::random(rng);
    Fp2 b = Fp2::random(rng);
    Fp2 c = Fp2::random(rng);
    // Addition + multiplication commute
    ASSERT_EQ(a + b, b + a);
    ASSERT_EQ(a * b, b * a);
    // Addition + multiplication are associative
    ASSERT_EQ(a + (b + c), (a + b) + c);
    ASSERT_EQ(a * (b * c), (a * b) * c);
    // Distributive propertly
    ASSERT_EQ(a * (b + c), a * b + a * c);
    // Inverses
    if (a != Fp2(0)) {
      ASSERT_EQ(inv(a) * a, Fp2(1));
    }
    ASSERT_EQ(Fp2(0) - a, -a);
    ASSERT_EQ(a + (-a), Fp2(0));
  }
}

} // namespace risc0
