use std::marker::PhantomData;

use polyglot_macro::{java_constructor, java_method};
use crate::types::jtypes::*;

use crate::ruesti::{Pass, Receive, Value};

pub struct Object {
    ptr: *mut Value,
}

unsafe impl Pass for Object {}
unsafe impl Receive for Object {
    fn from_polyglot_value(value: *mut Value) -> Self {
        Self { ptr: value }
    }
}

pub struct ArrayList<E>
where
    E: Pass + Receive,
{
    ptr: *mut Value,
    phantom: PhantomData<E>,
}

impl<E> ArrayList<E>
where
    E: Pass + Receive,
{
    java_constructor! {
        java.util.ArrayList<E> new_with_capacity(int initialCapacity);
        java.util.ArrayList<E> new();
    }
    java_method! {
        void trimToSize();
        void ensureCapacity(int minCapacity);
        int size();
        boolean isEmpty();
        boolean contains(Object o);
        int indexOf(Object o);
        int lastIndexOf(Object o);
        Object clone();
        E get(int index);
        E set(int index, E element);
        boolean add(E e);
        void add_at(int index, E element);
        E remove(int index);
        boolean remove_value(Object o);
        void clear();
    }
}

unsafe impl<T> Receive for ArrayList<T>
where
    T: Pass + Receive,
{
    fn from_polyglot_value(value: *mut Value) -> Self {
        Self {
            ptr: value,
            phantom: PhantomData,
        }
    }
}

unsafe impl<T> Pass for ArrayList<T> where T: Pass + Receive {}