extern crate alloc;

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::copy_nonoverlapping;

/// выровненный слайс
#[repr(align(16))]
pub struct Aligned<'a, T>
{
    data: &'a [T],
}

impl<'a, T> Aligned<'a, T>
{
    /// аллоцировать память в выровненном блоке и переместить туда данные
    #[inline(never)]
    pub fn from(source: &[T]) -> Self
    {
        let len = source.len();

        unsafe {
            let data = alloc(Self::layout(len)) as *mut T;

            copy_nonoverlapping(source as *const [T] as *const T, data, len);

            Self {
                data: core::slice::from_raw_parts(data, len),
            }
        }
    }

    fn layout(length: usize) -> Layout
    {
        if size_of::<T>() >= 4 {
            return Layout::array::<T>(length).unwrap();
        }

        let size = size_of::<T>() * length;
        let length = (size + 7) / 8;

        Layout::array::<u64>(length).unwrap()
    }
}

impl<'a, T> Deref for Aligned<'a, T>
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &'a Self::Target
    {
        self.data
    }
}

impl<'a, T> Drop for Aligned<'a, T>
{
    #[inline(never)]
    fn drop(&mut self)
    {
        unsafe {
            let layout = Self::layout(self.data.len());
            dealloc(self.data.as_ptr() as *mut u8, layout)
        }
    }
}
