use crate::klib::linked_list::{RawLinkedList, RawLinkedListNode};
use crate::memory::map_frame_identity;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Page, Size4KiB};

const fn pool_size<T: 'static>() -> usize {
    (4096 - size_of::<PoolInfo<T>>()) / size_of::<T>()
}

const fn pool_padding<T: 'static>() -> usize {
    (4096 - size_of::<PoolInfo<T>>()) % size_of::<T>()
}

#[repr(C, align(16))]
struct PoolInfo<T: 'static> {
    valid: u16,
    next: PoolPage<T>,
}

#[repr(transparent)]
#[derive(Copy)]
struct PoolPage<T: 'static> {
    ptr: *mut MaybeUninit<T>,
}

impl<T: 'static> Clone for PoolPage<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T: 'static> PoolPage<T> {
    unsafe fn alloc_page(page_allocator: &mut impl FrameAllocator<Size4KiB>) -> Self {
        let new_page = page_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for");
        let new_page = map_frame_identity(new_page);
        let new_ppage = unsafe { Self::setup_page(new_page) };
        new_ppage
    }

    unsafe fn pool_info(&self) -> &'static mut PoolInfo<T> {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::from_ptr(self.ptr));
        let info = &mut *((page.start_address() + page.size() - size_of::<PoolInfo<T>>() as u64)
            .as_mut_ptr() as *mut PoolInfo<T>);
        info
    }

    unsafe fn setup_page(page: Page<Size4KiB>) -> PoolPage<T> {
        let ppage = Self {
            ptr: page.start_address().as_mut_ptr(),
        };

        let info = ppage.pool_info();
        info.next = PoolPage {
            ptr: core::ptr::null_mut(),
        };
        info.valid = 0;
        ppage
    }

    unsafe fn is_valid(&self, i: u16) -> bool {
        self.pool_info().valid > i // if i <= pool_info.valid then the value has never been used and may contain garbage data.
    }

    /// This function **will** produce UB if `self.pool_info.valid >= self.elements.len()`
    ///
    /// This function will take the next value in the pool, mark it as valid, and continue.
    unsafe fn take_next_unchecked(&mut self) -> &'static mut MaybeUninit<T> {
        let i = self.pool_info().valid;
        self.pool_info().valid += 1;
        self.element(i as usize)
    }

    unsafe fn element(&self, i: usize) -> &'static mut MaybeUninit<T> {
        unsafe { &mut *self.ptr.offset(i as isize) }
    }

    #[inline]
    fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    ///
    /// Returns both the reference and the page containing it (since this is forward only, this improves performance since we then store that in the main pool).
    ///
    unsafe fn take_next(
        &mut self,
        page_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> (&'static mut MaybeUninit<T>, Self) {
        unsafe {
            let info = self.pool_info();
            if info.valid >= pool_size::<T>() as u16 {
                if info.next.is_null() {
                    let mut new_ppage = Self::alloc_page(page_allocator);
                    let elem = new_ppage.take_next_unchecked();
                    info.next = new_ppage;
                    (elem, info.next.clone()) // we can actually do this since we know that this page is the latest
                } else {
                    info.next.take_next(page_allocator)
                }
            } else {
                (self.take_next_unchecked(), self.clone())
            }
        }
    }

    unsafe fn next_page(&self) -> Self {
        unsafe { self.pool_info().next.clone() }
    }
}

///
/// Simple dynamic pool which holds a single-ended linked list of memory pages which it uses for an object.
/// There is no free operation, this is just here to do dynamic allocation of memory pages, mainly for allocator implementations (where no free is fine since we want to be prepared for worst case anyway).
///
pub struct PagedPool<T: 'static> {
    first_page: PoolPage<T>,
    active_page: PoolPage<T>,
}

impl<T: 'static> PagedPool<T> {
    pub fn new(page_allocator: &mut impl FrameAllocator<Size4KiB>) -> Self {
        let first_page = unsafe { PoolPage::<T>::alloc_page(page_allocator) };

        Self {
            first_page: PoolPage {
                ptr: first_page.ptr,
            },
            active_page: first_page,
        }
    }

    pub fn alloc(&mut self, page_allocator: &mut impl FrameAllocator<Size4KiB>) -> &'static mut T {
        let (r, page) = unsafe { self.active_page.take_next(page_allocator) };
        self.active_page = page.clone();
        unsafe { &mut *r.as_mut_ptr() }
    }

    pub fn iter_pages(&self) -> PagedPoolIterator<T> {
        PagedPoolIterator {
            page: self.first_page.clone(),
        }
    }
}

struct PagedPoolIterator<T: 'static> {
    page: PoolPage<T>,
}

impl<T: 'static> Iterator for PagedPoolIterator<T> {
    type Item = PoolPage<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let page = self.page.clone();
        if page.is_null() {
            None
        } else {
            self.page = unsafe { self.page.next_page() };
            Some(page)
        }
    }
}

pub struct PoolAllocator<T: 'static> {
    page_alloc: PagedPool<RawLinkedListNode<T>>,
    unused: RawLinkedList<T>,
}

impl<T: 'static> PoolAllocator<T> {
    pub fn new(frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Self {
        Self {
            page_alloc: PagedPool::new(frame_allocator),
            unused: RawLinkedList::new(),
        }
    }

    pub fn alloc(
        &mut self,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> &'static mut RawLinkedListNode<T> {
        if let Some(node) = self.unused.pop_front() {
            node
        } else {
            self.page_alloc.alloc(frame_allocator)
        }
    }

    pub fn get_pool(&self) -> &PagedPool<RawLinkedListNode<T>> {
        &self.page_alloc
    }
}
