# IntRangeExt

`IntRangeExt` is a trait that implements some convenience methods for integer ranges.
integer types could be: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`.


# Usage example

```rust
use int_range_ext::IntRangeExt;

assert_eq!((0..10).to_inclusive(), Ok(0..=9));
assert_eq!((0..).to_inclusive(), Ok(0..=i32::MAX));

assert_eq!((0..10).contains_subrange(&(1..8)), Ok(true));

assert!((0..10).equal(&(0..=9)));
assert!((..).equal(&(0u8..=255)));

assert_eq!((0..10).substract(&(4..=7)), Ok((Some(0..=3), Some(8..=9))));

assert_eq!((0..10).intersect(&(0..=7)), Ok(true));
```

