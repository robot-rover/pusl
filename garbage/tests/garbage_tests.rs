extern crate garbage;

use garbage::{Gc, ManagedPool, MarkTrace};
use std::cell::{RefCell};
use std::fmt;
use std::fmt::Formatter;

use std::rc::Rc;

#[derive(Clone)]
struct DropNotify(
    i32,
    Option<Gc<RefCell<DropNotify>>>,
    Rc<RefCell<Vec<i32>>>,
);

impl DropNotify {
    fn new(data: i32, drop_log: Rc<RefCell<Vec<i32>>>) -> Self {
        println!("Created #{}", data);
        DropNotify(data, None, drop_log)
    }

    fn set_ptr(&mut self, ptr: Gc<RefCell<DropNotify>>) {
        self.1 = Some(ptr)
    }
}

impl fmt::Debug for DropNotify {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DropNotify(#{}, &{})",
            self.0,
            self.1.as_ref().map(|ptr| ptr.borrow().0).unwrap_or(-1)
        )
    }
}

impl Drop for DropNotify {
    fn drop(&mut self) {
        self.2.borrow_mut().push(self.0);
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
    let drop_log = Rc::new(RefCell::new(Vec::new()));
    let mut pool = ManagedPool::new();
    let data1 = DropNotify::new(1, drop_log.clone());
    let data2 = DropNotify::new(2, drop_log.clone());
    let data3 = DropNotify::new(3, drop_log.clone());
    let ptr1 = pool.place_in_heap(RefCell::from(data1));
    let ptr2 = pool.place_in_heap(RefCell::from(data2));
    let ptr3 = pool.place_in_heap(RefCell::from(data3));

    ptr1.borrow_mut().set_ptr(ptr2.clone());
    ptr2.borrow_mut().set_ptr(ptr3.clone());

    println!("{}", ptr1.borrow().0);

    let anchors: Vec<Gc<dyn MarkTrace>> = vec![ptr2 as Gc<dyn MarkTrace>];
    println!("{}", ptr1.borrow().0);
    pool.collect_garbage(anchors.iter());
    println!("{}", ptr1.borrow().0);
    assert_eq!(&*drop_log.borrow(), &vec![1]);
    println!("{}", ptr1.borrow().0);

    pool.collect_garbage(std::iter::empty());
    assert_eq!(&*drop_log.borrow(), &vec![1, 2, 3]);

    println!("Done");
}
