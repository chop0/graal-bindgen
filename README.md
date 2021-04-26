# Safe Rust <---> GraalVM bindings
epic readme and examples coming soon.
arraylist example:
```rust
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
```

Make sure you change the `ar` and `linker` keys in .config/cargo.toml to point to your Graal LLVM toolchain.
