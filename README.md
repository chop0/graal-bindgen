# `graal-bindgen` <!-- omit in toc -->
## `graal-bindgen` generates safe bindings between Rust and Graal Polyglot so that you can use Java types and methods as if they were native. <!-- omit in toc -->
- [Overview](#overview)
- [Building](#building)
- [Constructor stubs](#constructor-stubs)
- [Function stubs](#function-stubs)
- [Pass and Receive](#pass-and-receive)
  - [Pass and Passable](#pass-and-passable)
  - [Receive](#receive)
- [Generics](#generics)
- [Arrays](#arrays)
- [ArrayList example](#arraylist-example)

## Overview
The `class` macro is the primary way to generate bindings to Java types;  it will generate a `struct` (with generics if specified) that implements `Pass` and `Receive` and has all the methods you give stubs for.  The methods generated can be used like normal rust methods, however mutability is **not** enforced.  The fully-qualified type name should precede a block containing method and constructor stubs.  Java primitives like `char`, `int`, and `byte` are aliased to corresponding Rust types.  

## Building
First, make sure you have [`cargo-make`](https://github.com/sagiegurari/cargo-make) installed, the `GRAAL_HOME` environment variable points to the root directory of your GraalVM installation, and the GraalVM LLVM toolchain is installed:
```bash
export GRAAL_HOME=[PATH_TO_GRAAL]
cargo install cargo-make
${GRAAL_HOME}/bin/gu install llvm-toolchain
```
You can then run
```bash
cargo make run
```
to run `main.rs`, or
```bash
cargo make build
```
to just compile it.

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

## Arrays
Arrays are represented by `JavaArray`.  Currently, creating and updating elements in them has not been implemented and `Index` cannot be implemented, since the trait requires a reference to be returned.  The return value of .get() is an `Option`;  if the index is out of bounds, it will be `None`, otherwise it will be `Some(value_at_index)`.

## ArrayList example
Using java `ArrayList`s and arrays:
```rust
use crate::polyglot::*;
use std::marker::PhantomData;
use crate::types::jtypes::*;

polyglot_macro::class! [java.util.ArrayList<E> {
    new();
    E get(int index);
    boolean add(E e);
    E[] toArray();
}];

let list = ArrayList::new();
let list_in_list = ArrayList::new();
for i in 0..100 {
    list_in_list.add(i);
}
list.add(list_in_list);
let array_from_list = list.get(0).toArray();
for i in 0..100 {
    println!("{}", array_from_list.get(i).unwrap());
}
```
Equivalent using `Vec` and slices:
```rust
let mut vec = Vec::new();
let mut vec_in_vec = Vec::new();
for i in 0..100 {
    vec_in_vec.push(i);
}
vec.push(vec_in_vec);
let slice_from_vec = vec.get(0).unwrap().as_slice();
for i in 0..100 {
    println!("{}", slice_from_vec.get(i).unwrap());
}
```

A full implementation of `java.util.ArrayList` can be seen in [src/builtins/mod.rs](src/builtins/mod.rs).