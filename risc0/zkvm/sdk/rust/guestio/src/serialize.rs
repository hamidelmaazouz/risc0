extern crate alloc;

use alloc::vec::Vec;

use super::{align_bytes_to_words, as_words_padded, Result, pad_words};
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

pub struct Alloc<'a> {
    buf_left: MaybeUninit<&'a mut [u32]>,
    base: usize,
}

pub struct AllocBuf<'a> {
    buf: &'a mut [u32],
    base: usize,
}

impl<'a> Alloc<'a> {
    #[track_caller]
    fn alloc(&mut self, words: usize) -> AllocBuf<'a> {
        let mut tmp : MaybeUninit<&'a mut [u32]> = MaybeUninit::uninit();
        std::mem::swap(&mut self.buf_left, &mut tmp);
        let buf_left = unsafe {tmp.assume_init()};

        assert!(buf_left.len() >= words);
        let (new_buf, rest) = buf_left.split_at_mut(words);
        self.buf_left.write(rest);
        let old_base = self.base;
        self.base += words;
        AllocBuf {
            buf: new_buf,
            base: old_base,
        }
    }
}

impl<'a> AllocBuf<'a> {
    pub fn descend(&mut self, offset: usize) -> AllocBuf {
        AllocBuf {
            buf: &mut self.buf[offset..],
            base: self.base + offset,
        }
    }

    pub fn rel_ptr_from(&self, other: &AllocBuf) -> u32 {
        (self.base - other.base) as u32
    }
}

impl<'a> Index<usize> for AllocBuf<'a> {
    type Output = u32;

    fn index(&self, offset: usize) -> &u32 {
        &self.buf[offset]
    }
}
impl<'a> IndexMut<usize> for AllocBuf<'a> {
    fn index_mut(&mut self, offset: usize) -> &mut u32 {
        &mut self.buf[offset]
    }
}

pub fn serialize<T>(val: &T) -> Result<Vec<u32>>
where
    T: Serialize,
{
    let tot_len = val.tot_len();
    let padded = pad_words(tot_len);
    let mut buf: Vec<u32> = Vec::new();
    buf.resize(padded, 0);

    let (mut alloc_buf, _) = buf.as_mut_slice().split_at_mut(tot_len);

    let mut alloc = Alloc {
        buf_left: MaybeUninit::new(&mut alloc_buf),
        base: 0,
    };

    let mut fixed = alloc.alloc(T::fixed_len());
    val.fill(&mut fixed, &mut alloc)?;

    Ok(buf)
}

pub trait Serialize {
    const FIXED_WORDS: usize;

    fn fixed_len() -> usize {
        Self::FIXED_WORDS
    }
    fn tot_len(&self) -> usize;

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()>;
}

impl Serialize for u32 {
    const FIXED_WORDS: usize = 1;

    fn tot_len(&self) -> usize {
        1
    }

    fn fill(&self, buf: &mut AllocBuf, _a: &mut Alloc) -> Result<()> {
        buf[0] = *self;
        Ok(())
    }
}

impl Serialize for std::string::String {
    const FIXED_WORDS: usize = 2;

    fn tot_len(&self) -> usize {
        Self::FIXED_WORDS + align_bytes_to_words(self.len())
    }

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()> {
        let words = align_bytes_to_words(self.len());
        let str_data = a.alloc(words);

        for (w, val) in std::iter::zip(
            str_data.buf.iter_mut(),
            as_words_padded(self.as_bytes().into_iter().cloned()),
        ) {
            *w = val;
        }

        buf[0] = self.len() as u32;
        buf[1] = str_data.rel_ptr_from(buf);
        Ok(())
    }
}

impl<T: Serialize> Serialize for std::option::Option<T> {
    const FIXED_WORDS: usize = 1;
    fn fixed_len() -> usize {
        1
    }

    fn tot_len(&self) -> usize {
        match self {
            None => 1,
            Some(val) => 1 + val.tot_len(),
        }
    }

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()> {
        match self {
            None => {
                buf[0] = 0;
                Ok(())
            }
            Some(val) => {
                let mut sub_buf = a.alloc(T::fixed_len());
                val.fill(&mut sub_buf, a)?;
                buf[0] = sub_buf.rel_ptr_from(buf);
                Ok(())
            }
        }
    }
}

impl<T: Serialize> Serialize for std::boxed::Box<T> {
    const FIXED_WORDS: usize = T::FIXED_WORDS;
    fn fixed_len() -> usize {
        T::fixed_len()
    }

    fn tot_len(&self) -> usize {
        self.as_ref().tot_len()
    }

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()> {
        self.as_ref().fill(buf, a)
    }
}

impl<T: Serialize> Serialize for std::vec::Vec<T> {
    const FIXED_WORDS: usize = 2;

    fn tot_len(&self) -> usize {
        self.iter().map(|x| x.tot_len()).sum::<usize>() + Self::FIXED_WORDS
    }

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()> {
        let mut sub_buf = a.alloc(T::fixed_len() * self.len());
        let mut pos = 0;
        for val in self {
            val.fill(&mut sub_buf.descend(pos), a)?;
            pos += T::fixed_len();
        }
        buf[0] = self.len() as u32;
        buf[1] = sub_buf.rel_ptr_from(buf);
        Ok(())
    }
}

impl<T: Serialize, const N: usize> Serialize for [T; N] {
    const FIXED_WORDS: usize = T::FIXED_WORDS * N;

    fn tot_len(&self) -> usize {
        self.iter().map(|x| x.tot_len()).sum()
    }

    fn fill(&self, buf: &mut AllocBuf, a: &mut Alloc) -> Result<()> {
        let mut pos = 0;
        for val in self {
            val.fill(&mut buf.descend(pos), a)?;
            pos += T::fixed_len();
        }
        Ok(())
    }
}
