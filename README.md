
# Parse Helper

This is a library full of primitives to slice up a string or byte slice.
My own primary use for it is when I'm writing a hand-crafted parser. 
These are the primitives that are useful when you're doing that.

Essentially, a [`ParseHelper`](ParseHelper) forms an api for an iterator over a sequence of bytes.
However, this iterator doesn't even implement rust's default [`Iterator`] trait.
Instead, the methods are more nuanced than `next()`, and do more specific things useful when dividing up strings.

## TODO:

* Support for custom tokens instead of either utf-8 characters or bytes
