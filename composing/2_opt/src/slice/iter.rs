use core::marker::PhantomData;
use core::slice::from_raw_parts;

#[repr(align(16))]
pub struct CharsIter<'a>
{
    ptr: *const u8,
    end: *const u8,
    breakpoint: *const u8,
    _marker: PhantomData<&'a u8>,
}

impl<'a> CharsIter<'a>
{
    #[inline(always)]
    pub fn new(str: &'a str) -> Self
    {
        unsafe {
            let length = str.len() as isize;
            let ptr = str.as_ptr();
            let end = ptr.offset(length);

            Self {
                ptr,
                breakpoint: ptr,
                end,
                _marker: PhantomData,
            }
        }
    }

    /// запомнить текущую позицию
    #[inline(always)]
    pub fn set_breakpoint(&mut self)
    {
        self.breakpoint = self.ptr;
    }

    /// указатель на запомненной позиции?
    #[inline(always)]
    pub fn at_breakpoint(&mut self, offset: isize) -> bool
    {
        unsafe { self.ptr.offset_from(self.breakpoint) - offset == 0 }
    }

    /// данные закончились?
    #[inline(always)]
    pub fn is_empty(&self) -> bool
    {
        unsafe { self.end.offset_from(self.ptr) == 0 }
    }

    /// прочитать байт без проверки длины оставшихся данных
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> &'a u8
    {
        let old = self.ptr;
        self.ptr = unsafe { self.ptr.add(1) };
        &*old
    }

    /// конечный участок слайса от запомненной позиции
    #[inline]
    pub fn ending_slice(&self) -> &[u8]
    {
        unsafe {
            let length = self.end.offset_from(self.breakpoint) as usize;
            from_raw_parts(self.breakpoint, length)
        }
    }

    /// слайс от запомненной позиции до текущего указателя, минус поправка
    #[inline]
    pub fn block_slice(&self, offset: isize) -> &[u8]
    {
        unsafe {
            let length = (self.ptr.offset_from(self.breakpoint) - offset) as usize;
            from_raw_parts(self.breakpoint, length)
        }
    }
}
