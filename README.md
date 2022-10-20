# front-vec
Efficiently-prependable Vec and String types.

Exposes two types: `FrontVec<T>` and `FrontString`. Each implements a `push_front` method which efficiently prepends a value onto the *front* of the data structure.

The types implement `Deref` so that they can be used like slices/`&str`s.

# `unsafe` Warning
This is an "in-development" crate. I'm not certain all uses of `unsafe` are valid yet. Please don't use this for anything important yet.

## Potential Use Cases
### Efficient **Cons**-Lists
My use-case is for packing data together in memory more eficiently than a cons-list, but with the same API as one.

## Representation in Memory
```
my_front_vec =
[len: usize     = 3]
[cap: usize     = 8]
[buf: Unique<T> = *]
                  |
                  |
                  v
[?, ?, ?, ?, ?, x1, x2, x3]
```

This diagram shows the memory representation of a `FrontVec<T>` which corresponds to the the `Vec` `vec![x1, x2, x3]`. The question marks (`?`) represent uninitialized data.

### Downsides
This representation (I believe) does not allow the use of the `realloc` function, which assumes memory at the front of the buffer is initialized. So a `Front{Vec,String}` is slightly less efficient because of that.

## Disclaimer
This is my first time writing `unsafe` code, so any safety audit contributions are certainly welcome!