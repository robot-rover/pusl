extern crate garbage;

use garbage::{GcPointer, ManagedPool, MarkTrace};
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct DropNotify(i32, Option<GcPointer<RefCell<DropNotify>>>);

impl DropNotify {
    fn new(data: i32) -> Self {
        println!("Created #{}", data);
        DropNotify(data, None)
    }

    fn set_ptr(&mut self, ptr: GcPointer<RefCell<DropNotify>>) {
        self.1 = Some(ptr)
    }
}

impl Drop for DropNotify {
    fn drop(&mut self) {
        println!("Dropped #{}", self.0)
    }
}

impl MarkTrace for DropNotify {
    fn mark_children(&self) {
        println!("#{} Marking Children", self.0);
        if let Some(ptr) = &self.1 {
            ptr.mark_recurse()
        }
    }
}

#[test]
fn basic_gc_test() {
    let mut pool = ManagedPool::new();
    let data1 = DropNotify::new(1);
    let data2 = DropNotify::new(2);
    let data3 = DropNotify::new(3);
    let ptr1 = pool.place_in_heap(RefCell::from(data1));
    let ptr2 = pool.place_in_heap(RefCell::from(data2));
    let ptr3 = pool.place_in_heap(RefCell::from(data3));

    ptr1.borrow_mut().set_ptr(ptr2.clone());
    ptr2.borrow_mut().set_ptr(ptr3.clone());
    println!("{}", (*ptr1).borrow().0);

    println!("{:?} | {:?}", ptr1.borrow(), ptr2.borrow());

    let anchors: Vec<GcPointer<dyn MarkTrace>> = vec![ptr2.into()];
    pool.collect_garbage(anchors.iter());

    println!("Done")
}
