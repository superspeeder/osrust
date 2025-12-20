use core::marker::PhantomData;
use core::ops::{Deref, DerefMut, DerefPure};

///
/// Linked list which uses no allocator.
/// Requires that the user manages allocation.
///
pub struct RawLinkedList<T> {
    head: *mut RawLinkedListNode<T>,
    tail: *mut RawLinkedListNode<T>,
}

///
/// # Safety
/// If the `next` and `prev` pointers are not null, then they must point to valid linked list nodes.
///
pub struct RawLinkedListNode<T> {
    pub value: T,
    pub(self) next: *mut RawLinkedListNode<T>,
    pub(self) prev: *mut RawLinkedListNode<T>,
}

impl<T> Deref for RawLinkedListNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for RawLinkedListNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

unsafe impl<T> DerefPure for RawLinkedListNode<T> {}

impl<T> RawLinkedListNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }

    ///
    /// Link this node after the provided node
    ///
    /// i.e.
    /// a -> b
    /// c.link_after(a)
    /// a -> c -> b
    ///
    pub unsafe fn link_after(&mut self, node: &mut Self) {
        self.next = node.next;
        self.prev = node as *mut Self;
        node.next = self as *mut Self;

        if !self.next.is_null() {
            unsafe {
                (*self.next).prev = self as *mut Self;
            }
        }
    }

    #[inline]
    pub unsafe fn link_before(&mut self, node: &mut Self) {
        unsafe {
            node.link_after(self);
        }
    }

    pub unsafe fn unlink(&mut self) {
        if !self.prev.is_null() {
            unsafe {
                (*self.prev).next = self.next;
            }
        }
        if !self.next.is_null() {
            unsafe {
                (*self.next).prev = self.prev;
            }
        }

        self.next = core::ptr::null_mut();
        self.prev = core::ptr::null_mut();
    }

    pub unsafe fn has_cycle(&self) -> bool {
        let mut a: *const _ = self as *const Self;
        let mut b: *const _ = self.next as *const Self;
        while !a.is_null() && !b.is_null() {
            a = (*a).next as *const _;
            if !a.is_null() {
                b = (*a).next as *const _;
            }

            if a == b {
                return true;
            }
        }

        false
    }
}

impl<T> RawLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
        }
    }

    pub fn prepend(&mut self, node: &mut RawLinkedListNode<T>) {
        if self.head.is_null() {
            self.head = node as *mut RawLinkedListNode<T>;
            self.tail = self.head;
        } else {
            unsafe { (*self.head).link_before(node) };
            self.head = node as *mut RawLinkedListNode<T>;
        }
    }

    pub fn append(&mut self, node: &mut RawLinkedListNode<T>) {
        if self.tail.is_null() {
            self.tail = node as *mut RawLinkedListNode<T>;
            self.head = self.tail;
        } else {
            unsafe { (*self.tail).link_after(node) };
            self.tail = node as *mut RawLinkedListNode<T>;
        }
    }

    pub fn pop_front(&mut self) -> Option<&mut RawLinkedListNode<T>> {
        if self.head.is_null() { None }
        else {
            unsafe {
                let rv = Some(&mut *self.head);
                if (&*self.head).next.is_null() {
                    self.head = core::ptr::null_mut();
                    self.tail = core::ptr::null_mut();
                } else {
                    self.head = (&*self.head).next;
                }
                rv.as_ref().unwrap_unchecked().unlink();
                rv
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &RawLinkedListNode<T>> {
        if self.head.is_null() {
            InvalidIter { _phantom: PhantomData }
        } else {
            LinkedListIter { node: &self.head }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RawLinkedListNode<T>> {
        if self.head.is_null() {
            InvalidIter { _phantom: PhantomData }
        } else {
            LinkedListMutIter { node: &mut self.head }
        }
    }

    pub unsafe fn front(&self) -> &'static RawLinkedListNode<T> {
        unsafe { &*self.head }
    }

    pub unsafe fn front_mut(&self) -> &'static mut RawLinkedListNode<T> {
        unsafe { &mut *self.head }
    }

    pub unsafe fn end(&self) -> &'static RawLinkedListNode<T> {
        unsafe { &*self.tail }
    }

    pub unsafe fn end_mut(&self) -> &'static mut RawLinkedListNode<T> {
        unsafe { &mut *self.tail }
    }
}

pub struct InvalidIter<T> {
    _phantom: PhantomData<T>,
}

impl<T> Iterator for InvalidIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct LinkedListIter<'a, T: 'static> {
    node: &'a RawLinkedListNode<T>,
}

impl<'a, T: 'static> Iterator for LinkedListIter<'a, T> {
    type Item = &'a RawLinkedListNode<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.node.next.is_null() {
            None
        } else {
            Some(unsafe { &*self.node.next })
        }
    }
}

pub struct LinkedListMutIter<'a, T: 'static> {
    node: &'a RawLinkedListNode<T>,
}

impl<'a, T: 'static> Iterator for LinkedListMutIter<'a, T> {
    type Item = &'a mut RawLinkedListNode<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.node.next.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.node.next })
        }
    }
}