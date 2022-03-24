use std::cell::{Cell, RefCell};
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;

#[derive(Debug)]
pub struct GcPointer<T: MarkTrace + ?Sized> {
    ptr: Cell<NonNull<ManagedData<T>>>,
    marker: PhantomData<Rc<T>>,
}

impl<T: MarkTrace + ?Sized + 'static> PartialEq for GcPointer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.get() == other.ptr.get()
    }
}

impl<T: MarkTrace + ?Sized + 'static> GcPointer<T> {
    pub fn mark_recurse(&self) {
        unsafe {
            let managed_box: &ManagedData<T> = &*self.ptr.get().as_ptr();
            if !managed_box.get_flag() {
                managed_box.set_flag(true);
                managed_box.data.mark_children();
            }
        }
    }

    fn new(ptr: NonNull<ManagedData<T>>) -> Self {
        GcPointer {
            ptr: Cell::new(ptr),
            marker: PhantomData,
        }
    }

    pub fn write_addr(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr.get().as_ptr())
    }
}

impl<T: MarkTrace + 'static> From<GcPointer<T>> for GcPointer<dyn MarkTrace> {
    fn from(concrete: GcPointer<T>) -> Self {
        GcPointer::new(concrete.ptr.get())
    }
}

impl<T: MarkTrace + 'static> Deref for GcPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let test = self.ptr.get().as_ptr();
        let t2 = unsafe { &*test };
        &t2.data
    }
}

impl<T: MarkTrace + ?Sized> Clone for GcPointer<T> {
    fn clone(&self) -> Self {
        GcPointer {
            ptr: Cell::new(self.ptr.get()),
            marker: PhantomData,
        }
    }
}

pub trait MarkTrace {
    /// Call mark_recurse on all children
    fn mark_children(&self);
}

impl<T: MarkTrace + ?Sized + 'static> MarkTrace for dyn Deref<Target = T> {
    fn mark_children(&self) {
        self.deref().mark_children();
    }
}

impl<T: MarkTrace + ?Sized + 'static> MarkTrace for RefCell<T> {
    fn mark_children(&self) {
        self.borrow().mark_children();
    }
}

impl MarkTrace for String {
    fn mark_children(&self) {}
}

impl<T: MarkTrace> MarkTrace for Vec<T> {
    fn mark_children(&self) {
        for item in self {
            item.mark_children();
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

    fn wrap_data<S: MarkTrace + 'static>(data: S) -> NonNull<ManagedData<S>> {
        let contents = ManagedData {
            flag: Cell::new(false),
            data,
        };
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(contents))) }
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

    pub fn place_in_heap<T: MarkTrace + 'static>(&mut self, data: T) -> GcPointer<T> {
        let managed_box = ManagedData::<T>::wrap_data(data);
        self.pool.push(managed_box);
        GcPointer::new(managed_box)
    }

    pub fn collect_garbage<'a, I>(&mut self, anchors: I)
    where
        I: IntoIterator<Item = &'a GcPointer<dyn MarkTrace>>,
    {
        println!("Recursive Marking");
        // For every rooted object, recursively mark all objects, stopping a branch if an object is already marked
        for anchor in anchors {
            let mut anchor = anchor.ptr.get();
            if !unsafe { anchor.as_ref() }.flag.get() {
                unsafe { anchor.as_mut() }.flag.set(true);
                unsafe { anchor.as_mut() }.data.mark_children();
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
                    Box::from_raw(nn_ptr.as_ptr());
                })
        }
    }
}

// Drop all managed objects in pool
impl Drop for ManagedPool {
    fn drop(&mut self) {
        self.pool.drain(..).for_each(|obj| unsafe {
            Box::from_raw(obj.as_ptr());
        })
    }
}
