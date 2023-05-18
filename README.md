# address offset as associated constant

Example of generated code:

```rs
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct Reg<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize>(
        PhantomData<(T, A)>,
    );
    unsafe impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Send
        for Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET>
    {
    }
    unsafe impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize> Sync
        for Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET>
    {
    }
    impl<T: Copy, A: Access, const BASE_ADDRESS: usize, const ADDRESS_OFFSET: usize>
        Reg<T, A, BASE_ADDRESS, ADDRESS_OFFSET>
    {
        const ADDRESS: usize = BASE_ADDRESS + ADDRESS_OFFSET;
        pub const fn new() -> Self {
            Self(PhantomData)
        }
        #[inline(always)]
        pub fn ptr(&self) -> *mut T {
            Self::ADDRESS as _
        }
    }
```

Currently, `BASE_ADDRESS` is not implemented.
It is generated as 0, not the one provided by peripheral node.
Here we see the correct address, 0xF0200000.


```rs
pub mod can0 {
    pub const BASE_ADDRESS: usize = 0xF0200000;
    #[doc = "ce_mc_m_can"]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Can0;
    unsafe impl Send for Can0 {}
    unsafe impl Sync for Can0 {}
    impl Can0 {
        #[doc = "CAN Clock Control Register"]
        #[inline(always)]
        pub const fn clc(
            self,
        ) -> crate::common::Reg<regs::Clc, crate::common::RW, BASE_ADDRESS, 32768usize> {
            crate::common::Reg::new()
        }
        #[doc = "Module Identification Register"]
        #[inline(always)]
        pub const fn id(
            self,
        ) -> crate::common::Reg<regs::Id, crate::common::R, BASE_ADDRESS, 32776usize> {
            crate::common::Reg::new()
        }
```

Use case:

```rs
use tc37x_pac as pac;

fn main() {
    let can = pac::can0::Can0;

    let mut clc = unsafe { can.clc().read() };
    let disr = clc.disr();

    if unsafe { disr.get() } {
        // ...
    }
}
```

Something even cleaner can be achieved, but:

```rs
pub mod can0 {
    use crate::common::*;

    #[doc = "ce_mc_m_can"]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Can0;
    unsafe impl Send for Can0 {}
    unsafe impl Sync for Can0 {}
    impl Can0 {
        #[doc = "CAN Clock Control Register"]
        #[inline(always)]
        pub const fn clc(self) -> Reg<regs::Clc, RW, { 0xF0200000 + 32768 }> {
            Reg::new()
        }
        #[doc = "Module Identification Register"]
        #[inline(always)]
        pub const fn id(self) -> Reg<regs::Id, R, { 0xF0200000 + 32776 }> {
            Reg::new()
        }
        #[doc = "Module Control Register"]
        #[inline(always)]
        pub const fn mcr(self) -> Reg<regs::Mcr, RW, { 0xF0200000 + 32816 }> {
            Reg::new()
        }
```
