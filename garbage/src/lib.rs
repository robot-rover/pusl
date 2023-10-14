#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(dispatch_from_dyn)]

use std::cell::{Cell, RefCell};
use std::fmt;
use std::fmt::Formatter;
use std::marker::{PhantomData, Unsize};
use std::ops::{CoerceUnsized, Deref, DispatchFromDyn};
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Gc<T: MarkTrace + ?Sized> {
    ptr: NonNull<ManagedData<T>>,
    marker: PhantomData<ManagedData<T>>,
}

// Needed to coerce Gc<RefCell<T>> -> Gc<RefCell<dyn Trait>> for T: Trait
impl<T: ?Sized + Unsize<U> + MarkTrace, U: ?Sized + MarkTrace> CoerceUnsized<Gc<U>> for Gc<T> {}
impl<T: ?Sized + Unsize<U> + MarkTrace, U: ?Sized + MarkTrace> DispatchFromDyn<Gc<U>> for Gc<T> {}

impl<T: MarkTrace + ?Sized + 'static> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T: MarkTrace + ?Sized + 'static> Gc<T> {
    fn new(ptr: NonNull<ManagedData<T>>) -> Self {
        Gc {
            ptr,
            marker: PhantomData,
        }
    }

    pub fn write_addr(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}

impl<T: MarkTrace + 'static> MarkTrace for Gc<T> {
    fn mark_trace(&self) {
        unsafe {
            let managed_box: &ManagedData<T> = self.ptr.as_ref();
            if !managed_box.get_flag() {
                managed_box.set_flag(true);
                managed_box.data.mark_trace();
            }
        }
    }
}

impl<T: MarkTrace + ?Sized + 'static> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().data }
    }
}

impl<T: MarkTrace + ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            ptr: self.ptr,
            marker: PhantomData,
        }
    }
}

pub trait MarkTrace {
    /// Call mark_trace on all children
    fn mark_trace(&self);
}

impl<T: MarkTrace + ?Sized + 'static> MarkTrace for dyn Deref<Target = T> {
    fn mark_trace(&self) {
        self.deref().mark_trace();
    }
}

impl<T: MarkTrace + ?Sized + 'static> MarkTrace for RefCell<T> {
    fn mark_trace(&self) {
        self.borrow().mark_trace();
    }
}

impl MarkTrace for String {
    fn mark_trace(&self) {}
}

impl<T: MarkTrace> MarkTrace for Vec<T> {
    fn mark_trace(&self) {
        for item in self {
            item.mark_trace();
        }
    }
}

struct ManagedData<T: MarkTrace + ?Sized> {
    flag: Cell<bool>,
    data: T,
}

impl<T: MarkTrace + ?Sized + 'static> ManagedData<T> {
    #[inline]
    fn set_flag(&self, val: bool) {
        self.flag.set(val)
    }
    #[inline]
    fn get_flag(&self) -> bool {
        self.flag.get()
    }
}

impl<T: MarkTrace + 'static> ManagedData<T> {
    fn wrap_data(data: T) -> NonNull<ManagedData<T>> {
        let contents = ManagedData {
            flag: Cell::new(false),
            data,
        };
        NonNull::new(Box::into_raw(Box::new(contents))).unwrap()
    }
}

#[derive(Debug)]
pub struct ManagedPool {
    pool: Vec<NonNull<ManagedData<dyn MarkTrace>>>,
}

impl ManagedPool {
    pub fn new() -> Self {
        ManagedPool { pool: Vec::new() }
    }

    pub fn place_in_heap<T: MarkTrace + 'static>(&mut self, data: T) -> Gc<T> {
        let managed_box: NonNull<ManagedData<T>> = ManagedData::<T>::wrap_data(data);
        let into_pool: NonNull<ManagedData<dyn MarkTrace>> = managed_box;
        self.pool.push(into_pool);
        Gc::new(managed_box)
    }

    pub fn collect_garbage<'a, I>(&mut self, anchors: I)
    where
        I: IntoIterator<Item = &'a Gc<dyn MarkTrace>>,
    {
        println!("Recursive Marking");
        // For every rooted object, recursively mark all objects, stopping a branch if an object is already marked
        for anchor in anchors {
            let anchor = unsafe { anchor.ptr.as_ref() };
            if !anchor.get_flag() {
                anchor.set_flag(true);
                anchor.data.mark_trace();
            }
        }

        println!("Dropping Unmarked");
        // Drop all non-marked objects, unmarking all objects in the process
        unsafe {
            let mut to_drop = Vec::new();
            self.pool.retain(|nn_ptr| {
                let obj = &*nn_ptr.as_ptr();
                let val = obj.flag.get();
                obj.flag.set(false);
                if !val {
                    to_drop.push(*nn_ptr);
                }
                val
            });

            to_drop.into_iter().for_each(|nn_ptr| {
                drop(Box::from_raw(nn_ptr.as_ptr()));
            })
        }
    }
}

impl Default for ManagedPool {
    fn default() -> Self {
        Self::new()
    }
}

// Drop all managed objects in pool
impl Drop for ManagedPool {
    fn drop(&mut self) {
        self.pool.drain(..).for_each(|obj| unsafe {
            drop(Box::from_raw(obj.as_ptr()));
        })
    }
}
