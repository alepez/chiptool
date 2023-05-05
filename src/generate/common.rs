use core::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct R;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct W;

mod sealed {
    use super::*;
    pub trait Access {}
    impl Access for R {}
    impl Access for W {}
    impl Access for RW {}
}

pub trait Access: sealed::Access + Copy {}
impl Access for R {}
impl Access for W {}
impl Access for RW {}

pub trait Read: Access {}
impl Read for RW {}
impl Read for R {}

pub trait Write: Access {}
impl Write for RW {}
impl Write for W {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Reg<T: Copy, A: Access> {
    ptr: *mut u8,
    phantom: PhantomData<*mut (T, A)>,
}
unsafe impl<T: Copy, A: Access> Send for Reg<T, A> {}
unsafe impl<T: Copy, A: Access> Sync for Reg<T, A> {}

impl<T: Copy, A: Access> Reg<T, A> {
    #[inline(always)]
    pub fn from_ptr(ptr: *mut u8) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        self.ptr as _
    }
}

impl<T: Copy, A: Read> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        (self.ptr as *mut T).read_volatile()
    }
}

impl<T: Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn write_value(&self, val: T) {
        (self.ptr as *mut T).write_volatile(val)
    }
}

impl<T: Default + Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

impl<T: Copy, A: Read + Write> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn modify<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = self.read();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

pub struct RegisterField<'a, const OFFSET: usize, const MASK: u32, V> {
    data: &'a mut u32,
    marker: PhantomData<V>
}

macro_rules! gen_field {
    ($field_type: ty) => {
        impl<'a, const OFFSET: usize, const MASK: u32> RegisterField<'a, OFFSET, MASK, $field_type> {
            pub(crate) fn from_register(data: &'a mut u32) -> Self {
                Self {
                    data,
                    marker: PhantomData
                }
            }

            pub fn get(&self) -> $field_type {
                let filtered = (*self.data >> OFFSET) & MASK;
                filtered.try_into().unwrap()
            }
            
            pub fn set(&mut self, value: $field_type) {
                let value = value as u32;
                *self.data &= !(MASK << OFFSET);
                *self.data |= value << OFFSET;
            }
        }
    }
}
gen_field!{u8}
gen_field!{u16}
gen_field!{u32}

impl<'a, const OFFSET: usize> RegisterField<'a, OFFSET, 1, bool> {
    pub(crate) fn from_register(data: &'a mut u32) -> Self {
        Self {
            data,
            marker: PhantomData
        }
    }

    pub fn get(&self) -> bool {
        let filtered = (*self.data >> OFFSET) & 1;
        filtered == 1
    }
    
    pub fn set(&mut self, value: bool) {
        let value = if value { 1u32 } else { 0u32 };
        *self.data &= !(1 << OFFSET);
        *self.data |= value << OFFSET;
    }
}
