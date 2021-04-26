> Rust to Graal Polyglot binding generator using procedural macros.
- [Overview](#overview)
- [Building](#building)
- [Constructor stubs](#constructor-stubs)
- [Function stubs](#function-stubs)
- [Pass and Receive](#pass-and-receive)
  - [Pass and Passable](#pass-and-passable)
  - [Receive](#receive)
- [Generics](#generics)

## Overview
The `class` macro is the primary way to generate bindings to Java types;  it will generate a `struct` (with generics if specified) that implements `Pass` and `Receive` and has all the methods you give stubs for.  The methods generated can be used like normal rust methods, however mutability is **not** enforced.  The fully-qualified type name should precede a block containing method and constructor stubs.  Java primitives like `char`, `int`, and `byte` are aliased to corresponding Rust types.  

## Building
Make sure you have [`cargo-make`](https://github.com/sagiegurari/cargo-make) installed.

## Constructor stubs
A stub is inferred to be a constructor if it doesn't have a return type.  The Rust name of the constructor must be explicitly declared, and doesn't have to be the name of the type.  
```java
class! [java.lang.String {
    new();
    new_from(String original);
}];
```
will expand to a struct `String` with the methods `String::new` and `String::new_from`:
```rust
pub fn new() -> String {
    let polyglot_type = crate::java_type("java.lang.String");
    String::from_polyglot_value({
        unsafe { crate::polyglot::internal::polyglot_new_instance(polyglot_type) }
    })
}
pub fn new_from(original: String) -> String {
    let polyglot_type = crate::java_type("java.lang.String");
    String::from_polyglot_value({
        unsafe {
            crate::polyglot::internal::polyglot_new_instance(
                polyglot_type,
                crate::polyglot::internal::expect_variadic(original),
            )
        }
    })
}
```

`String::new()` will be equivalent to calling `new String()` in Java, and `String::new_from(...)` will be equivalent to `new String(...)`.  

## Function stubs
Function stubs are composed of a return value, an optional alias, a name, and arguments.  
```<return type> [alias] <function_name> ( [<type> <arg_name>]* );```
Aliases make the Rust function name different to the function name in Java;  this is useful when you have an overloaded method since Rust does not support overloading and will not compile if two functions have the same name.  If left empty, the Rust binding name is presumed to be the same as the Java function name.
```java
class! [java.util.ArrayList<E> {
    int add_at add(int index, E element);
    int add(E element);
}];
```
will expand to a struct `ArrayList` with the methods `ArrayList::add_at` and `ArrayList::add`:
```rust
pub fn add_at(&self, index: int, element: E) -> int {
    return int::from_polyglot_value({
        unsafe {
            crate::polygot::internal::polyglot_invoke(
                self.ptr,
                crate::polygot::internal::make_cstring("add").as_ptr(),
                crate::polygot::internal::expect_variadic(index),
                crate::polygot::internal::expect_variadic(element),
            )
        }
    });
}
pub fn add(&self, element: E) -> int {
    return int::from_polyglot_value({
        unsafe {
            crate::polygot::internal::polyglot_invoke(
                self.ptr,
                crate::polygot::internal::make_cstring("add").as_ptr(),
                crate::polygot::internal::expect_variadic(element),
            )
        }
    });
}
```

## Pass and Receive
The `Pass` and `Receive` traits indicate that describe how a type can safely be passed to and received from Graal Polyglot.  

### Pass and Passable
The `Pass<T: Passable>` trait requires `fn pass(&self) -> T` to be implemented.  `Passable` is a marker trait for whether a type is safe to directly pass to polyglot, and because unboxed primitives are passed directly but all other types are passed using a pointer.  For all objects, `pass` should be implemented so that it returns its inner pointer (a `*Value`) like this:
```rust
fn pass(&self) -> *mut Value {
    self.ptr
}
```

### Receive
The `Receive` trait defines how a Polyglot value can be used to construct a type.  Usually, it should just instantiate a struct `Self` with its backing pointer set to the given `*mut Value` like this:
```rust
fn from_polyglot_value(value: *mut Value) -> Self {
    Self { ptr: value }
}
```

## Generics
`class!` supports generics;  the generic type must be `Pass + Receive`.  Due to the poor design decision of treating *mut Values and primitives differently (even though they can be passed to polyglot directly), `Pass` makes it so that there needs to be an extra parameter for each desired generic.  The first generic parameters are the ones you specify, followed by a `Passable` bound for each one you specified after.  Type inference should sort this out, but if you need to specify explicitly, you can tell Rust to still infer the Passable bounds like this:
```rust
let my_arraylist: ArrayList<i32, _> = ArrayList::new();
```
If something goes *really* wrong, you can explicitly specify the `Passable`.  For primitives, this will be the same as the main type.  For other Objects, this will be `*mut Value`.
```java
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