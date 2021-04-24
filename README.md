# Safe Rust <---> GraalVM bindings
epic readme and examples coming soon.
arraylist example:
```rust
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
```

this will be further macro-fied in the future so you don't need to use `unsafe` at all.  right now, the safe methods are not actually "safe", so buyer beware.