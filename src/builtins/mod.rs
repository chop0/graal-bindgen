use std::marker::PhantomData;

use crate::types::jtypes::*;
use graal_bindgen_macros::{class, java_constructor};

use crate::polyglot::{Pass, Receive, Value};

pub struct Object {
    ptr: *mut Value,
}

unsafe impl Pass<*mut Value> for Object {
    fn pass(&self) -> *mut Value {
        self.ptr
    }
}
unsafe impl Receive for Object {
    fn from_polyglot_value(value: *mut Value) -> Self {
        Self { ptr: value }
    }
}

pub struct String {
    ptr: *mut Value,
}

unsafe impl Pass<*mut Value> for String {
    fn pass(&self) -> *mut Value {
        self.ptr
    }
}
unsafe impl Receive for String {
    fn from_polyglot_value(value: *mut Value) -> Self {
        Self { ptr: value }
    }
}

impl String {
    java_constructor! {
        java.lang.String new();
    }
}

class! [java.util.ArrayList<E> {
    new_with_length(int initialCapacity);
    new();
    void trimToSize();
    void ensureCapacity(int minCapacity);
    int size();
    boolean isEmpty();
    boolean contains(Object o);
    int indexOf(Object o);
    int lastIndexOf(Object o);
    Object clone();
    E[] toArray();
    E get(int index);
    E set(int index, E element);
    boolean add(E e);
    void add_at add(int index, E element);
    E remove_at remove(int index);
    boolean remove_item remove(Object o);
    void clear();
   void removeRange(int fromIndex, int toIndex);
}];





