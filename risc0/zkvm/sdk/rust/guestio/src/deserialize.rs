use core::marker::PhantomData;
extern crate alloc;
use alloc::vec::Vec;
use impl_trait_for_tuples::impl_for_tuples;

pub trait Deserialize<'a> {
    type RefType;

    const FIXED_WORDS: usize;

    fn deserialize_from(words: &'a [u32]) -> Self::RefType;

    fn into_orig(val: Self::RefType) -> Self;
}

impl<'a> Deserialize<'a> for u32 {
    type RefType = u32;

    const FIXED_WORDS: usize = 1;

    fn deserialize_from(words: &[u32]) -> Self::RefType {
        words[0]
    }

    fn into_orig(val: Self::RefType) -> Self {
        val.into()
    }
}

impl<'a> Deserialize<'a> for std::string::String {
    type RefType = &'a str;

    const FIXED_WORDS: usize = 2;

    fn deserialize_from(words: &'a [u32]) -> Self::RefType {
        let (len, ptr) = (words[0], words[1]);

        std::str::from_utf8(&bytemuck::cast_slice(&words[ptr as usize..])[..len as usize]).unwrap()
    }
    fn into_orig(val: Self::RefType) -> Self {
        val.into()
    }
}

impl<'a, T: Deserialize<'a>> Deserialize<'a> for Option<T> {
    type RefType = Option<T::RefType>;

    const FIXED_WORDS: usize = 1;

    fn deserialize_from(words: &'a [u32]) -> Self::RefType {
        let ptr = words[0];

        if ptr == 0 {
            None
        } else {
            Some(T::deserialize_from(&words[ptr as usize..]))
        }
    }

    fn into_orig(val: Self::RefType) -> Self {
        val.map(|v| T::into_orig(v))
    }
}

impl<'a, T: Deserialize<'a>> Deserialize<'a> for Box<T> {
    type RefType = T::RefType;

    const FIXED_WORDS: usize = T::FIXED_WORDS;

    fn deserialize_from(words: &'a [u32]) -> T::RefType {
        T::deserialize_from(words)
    }

    fn into_orig(val: Self::RefType) -> Self {
        Box::new(T::into_orig(val))
    }
}

pub struct VecRef<'a, T> {
    len: usize,
    words: &'a [u32],
    phantom: PhantomData<T>,
}

impl<'a, T: Deserialize<'a>> VecRef<'a, T> {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn index(&self, index: usize) -> T::RefType {
        T::deserialize_from(&self.words[T::FIXED_WORDS * index..])
    }
}

pub struct VecRefIter<'a, T> {
    words: &'a [u32],
    items_left: usize,
    phantom: PhantomData<T>,
}

impl<'a, T: Deserialize<'a>> IntoIterator for VecRef<'a, T> {
    type Item = T::RefType;
    type IntoIter = VecRefIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VecRefIter {
            words: self.words,
            items_left: self.len,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: Deserialize<'a>> Iterator for VecRefIter<'a, T> {
    type Item = T::RefType;
    fn next(&mut self) -> Option<T::RefType> {
        if self.items_left > 0 {
            let val = T::deserialize_from(self.words);
            self.items_left -= 1;
            self.words = &self.words[T::FIXED_WORDS..];
            Some(val)
        } else {
            None
        }
    }
}

impl<'a, T: Deserialize<'a>> Deserialize<'a> for Vec<T> {
    type RefType = VecRef<'a, T>;

    const FIXED_WORDS: usize = 2;

    fn deserialize_from(words: &'a [u32]) -> Self::RefType {
        VecRef {
            len: words[0] as usize,
            words: &words[words[1] as usize..],
            phantom: PhantomData,
        }
    }

    fn into_orig(val: Self::RefType) -> Self {
        let mut v = Vec::with_capacity(val.len());
        v.extend(val.into_iter().map(|v| T::into_orig(v)));
        v
    }
}

impl<'a, T: Deserialize<'a>, const N: usize> Deserialize<'a> for [T; N] {
    type RefType = VecRef<'a, T>;

    const FIXED_WORDS: usize = N * T::FIXED_WORDS;

    fn deserialize_from(words: &'a [u32]) -> Self::RefType {
        VecRef {
            len: N,
            words,
            phantom: PhantomData,
        }
    }

    fn into_orig(val: Self::RefType) -> Self {
        match Vec::from_iter(val.into_iter().map(|x| T::into_orig(x))).try_into() {
            Ok(result) => result,
            _ => panic!("VecRef iterator didn't return the proper number of elements"),
        }
    }
}

#[impl_for_tuples(1, 5)]
impl<'a> Deserialize<'a> for Tuple {
    for_tuples!(type RefType = (#(Tuple::RefType),*););
    for_tuples!(const FIXED_WORDS: usize = #(Tuple::FIXED_WORDS)+*; );

    fn deserialize_from(words: &'a [u32]) -> Self::RefType {
        let mut pos = 0;
        let mut inc_pos = |n| {
            let old_pos = pos;
            pos += n;
            old_pos
        };
        for_tuples!(
            (#(
                Tuple::deserialize_from(&words[inc_pos(Tuple::FIXED_WORDS)..])
            ),*));
    }

    fn into_orig(val: Self::RefType) -> Self {
        for_tuples!(
            (#(
                Tuple::into_orig(val.Tuple)
            ),*));
    }
}

impl<'a> Deserialize<'a> for () {
    type RefType = ();
    const FIXED_WORDS: usize = 0;
    fn deserialize_from(_words: &'a [u32]) -> Self::RefType {
        ()
    }
    fn into_orig(_val: Self::RefType) -> Self {
        ()
    }
}
