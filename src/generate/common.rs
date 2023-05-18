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
pub struct Reg<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize>(PhantomData<(T, A)>);

unsafe impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Send for Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {}
unsafe impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Sync for Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {}

impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {
    const ADDRESS: usize = BASE_ADDRESS + ADDRESS_OFFSET;

    pub const fn new() -> Self {
        Self(PhantomData)
    }

    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        Self::ADDRESS as _
    }
}

impl<T: Copy, A: Read, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        (Self::ADDRESS as *mut T).read_volatile()
    }
}

impl<T: Copy, A: Write, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {
    #[inline(always)]
    pub unsafe fn write_value(&self, val: T) {
        (Self::ADDRESS as *mut T).write_volatile(val)
    }
}

impl<T: Default + Copy, A: Write, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {
    #[inline(always)]
    pub unsafe fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

impl<T: Copy, A: Read + Write, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET> {
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

            pub unsafe fn get(&self) -> $field_type {
                let filtered = (*self.data >> OFFSET) & MASK;
                filtered.try_into().unwrap()
            }
            
            pub unsafe fn set(&mut self, value: $field_type) {
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

    pub unsafe fn get(&self) -> bool {
        let filtered = (*self.data >> OFFSET) & 1;
        filtered == 1
    }
    
    pub unsafe fn set(&mut self, value: bool) {
        let value = if value { 1u32 } else { 0u32 };
        *self.data &= !(1 << OFFSET);
        *self.data |= value << OFFSET;
    }
}
