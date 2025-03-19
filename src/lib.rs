use core::ops::{Add, Bound, RangeBounds, RangeInclusive, Sub};


trait Utils {
    fn zero() -> Self;
    fn one() -> Self;
    fn max_() -> Self;
    fn min_() -> Self;
}

macro_rules! impl_one {
    ($($ident: ident),*) => {
        $(
            impl Utils for $ident {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn max_() -> Self { $ident::MAX }
                fn min_() -> Self { $ident::MIN }
            }
        )*
    };
}

impl_one!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);


trait IntRangeHelper<T> {
    fn is_empty(&self) -> bool;
    /// `self` must not be empty
    fn to_inclusive(&self) -> Result<RangeInclusive<T>, ()>;
    /// Both `self` and `other` must not be empty
    fn has<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<bool, ()>;
    fn equal<Other: RangeBounds<T> + Clone + IntRangeHelper<T>>(&self, other: &Other) -> bool 
        where Self: RangeBounds<T> + Clone
    {
        self.has(other).unwrap_or(false) && other.has(self).unwrap_or(false)
    }
    /// `self` must has `other`
    /// 
    /// ```no_run
    /// let r = 0..10;
    /// let (r1, r2) = r.sub(5..8);
    /// assert_eq!(r1, 0..=4);
    /// assert_eq!(r2, 8..=9);
    /// ```
    fn substract<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<(Option<RangeInclusive<T>>, Option<RangeInclusive<T>>), ()>;
    /// Both `self` and `other` must not be empty
    fn intersect<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<bool, ()>;
}

impl<T: Copy + Clone + PartialOrd + PartialEq + Ord + Eq + Add<Output = T> + Sub<Output = T> + Utils, U: RangeBounds<T> + Clone> IntRangeHelper<T> for U {
    fn is_empty(&self) -> bool {
        match self.start_bound() {
            Bound::Included(s) => {
                match self.end_bound() {
                    Bound::Included(e) => {
                        // [s, e]
                        !(s <= e)
                    },
                    Bound::Excluded(e) => {
                        // [s, e)
                        !(s < e)
                    },
                    Bound::Unbounded => {
                        // [s..
                        false
                    },
                }
            },
            Bound::Excluded(s) => {
                match self.end_bound() {
                    Bound::Included(e) => {
                        // (s, e]
                        !(s < e)
                    },
                    Bound::Excluded(e) => {
                        // (s, e)
                        !(s < e && *s + T::one() < *e)
                    },
                    Bound::Unbounded => {
                        // (s..
                        !(*s < T::max_())
                    },
                }
            },
            Bound::Unbounded => {
                match self.end_bound() {
                    Bound::Included(e) => {
                        // ..=e
                        !(T::min_() <= *e)
                    },
                    Bound::Excluded(e) => {
                        // ..e
                        !(T::min_() < *e)
                    },
                    Bound::Unbounded => {
                        // ..
                        false
                    },
                }
            },
        }
    }

    fn to_inclusive(&self) -> Result<RangeInclusive<T>, ()> {
        if self.is_empty() { return Err(()); }

        let s = match self.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n + T::one(),
            Bound::Unbounded => T::min_(),
        };

        let e = match self.end_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n - T::one(),
            Bound::Unbounded => T::max_(),
        };

        Ok(s ..= e)
    }

    fn has<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<bool, ()> {
        if self.is_empty() || other.is_empty() { return Err(()); }

        match other.start_bound() {
            Bound::Included(n) => {
                // [n..
                if !self.contains(n) { return Ok(false); }
            },
            Bound::Excluded(n) => {
                // (n..
                match self.start_bound() {
                    Bound::Included(x) => {
                        // (n..
                        // [x..
                        if x > n && *x > *n + T::one() { return Ok(false); }
                    },
                    Bound::Excluded(x) => {
                        // (n..
                        // (x..
                        if x > n { return Ok(false); }
                    },
                    Bound::Unbounded => {
                        // (n..
                        // ..
                    },
                }
            },
            Bound::Unbounded => {
                match self.start_bound() {
                    Bound::Included(n) => {
                        if *n != T::min_() { return Ok(false); }
                    },
                    Bound::Excluded(_) => { return Ok(false); },
                    Bound::Unbounded => {},
                }
            },
        }
        match other.end_bound() {
            Bound::Included(n) => {
                // ..=n
                if !self.contains(n) { return Ok(false); }
            },
            Bound::Excluded(n) => {
                // ..n
                match self.end_bound() {
                    Bound::Included(x) => {
                        // ..n
                        // ..=x
                        if x < n && *x + T::one() < *n { return Ok(false); }
                    },
                    Bound::Excluded(x) => {
                        // ..n
                        // ..x
                        if x < n { return Ok(false); }
                    },
                    Bound::Unbounded => {},
                }
            },
            Bound::Unbounded => {
                // ..
                match self.end_bound() {
                    Bound::Included(n) => {
                        if *n != T::max_() { return Ok(false); }
                    },
                    Bound::Excluded(_) => { return Ok(false); },
                    Bound::Unbounded => {},
                }
            },
        }

        Ok(true)
    }

    fn substract<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<(Option<RangeInclusive<T>>, Option<RangeInclusive<T>>), ()> {
        if !self.has(other).unwrap_or(false) { return Err(()); }

        // self.start .. other.start - 1
        let r1 = match self.start_bound() {
            Bound::Included(s) => {
                match other.start_bound() {
                    Bound::Included(e) => {
                        if s < e {
                            *s ..= *e - T::one()
                        } else {
                            T::one() ..= T::zero()
                        }
                    },
                    Bound::Excluded(e) => {
                        // [s..
                        // (e..
                        *s ..= *e
                    },
                    Bound::Unbounded => { T::one() ..= T::zero() },
                }
            },
            Bound::Excluded(s) => {
                // (s..
                match other.start_bound() {
                    Bound::Included(e) => {
                        // (s..
                        // [e..
                        *s + T::one() ..= *e - T::one()
                    },
                    Bound::Excluded(e) => {
                        // (s..
                        // (e..
                        *s + T::one() ..= *e
                    },
                    Bound::Unbounded => {
                        T::one() ..= T::zero()
                    },
                }
            },
            Bound::Unbounded => {
                match other.start_bound() {
                    Bound::Included(e) => {
                        if T::min_() < *e {
                            T::min_() ..= *e - T::one()
                        } else {
                            T::one() ..= T::zero()
                        }
                    },
                    Bound::Excluded(e) => {
                        T::min_() ..= *e
                    },
                    Bound::Unbounded => { T::one() ..= T::zero() },
                }
            },
        };

        // other.end .. self.end
        let r2 = match other.end_bound() {
            Bound::Included(s) => {
                if *s == T::max_() {
                    T::one() ..= T::zero()
                } else {
                    match self.end_bound() {
                        Bound::Included(e) => {
                            *s + T::one() ..= *e
                        },
                        Bound::Excluded(e) => {
                            *s + T::one() ..= *e - T::one()
                        },
                        Bound::Unbounded => {
                            *s + T::one() ..= T::max_()
                        },
                    }
                }
            },
            Bound::Excluded(s) => {
                match self.end_bound() {
                    Bound::Included(e) => {
                        *s ..= *e
                    },
                    Bound::Excluded(e) => {
                        *s ..= *e - T::one()
                    },
                    Bound::Unbounded => {
                        *s ..= T::max_()
                    },
                }
            },
            Bound::Unbounded => { T::one() ..= T::zero() },
        };

        let r1 = if r1.is_empty() {
            None
        } else {
            Some(r1)
        };

        let r2 = if r2.is_empty() {
            None
        } else {
            Some(r2)
        };

        return Ok((r1, r2));
    }

    fn intersect<Other: RangeBounds<T> + Clone>(&self, other: &Other) -> Result<bool, ()> {
        if self.is_empty() || other.is_empty() { return Err(()); }
        if self.has(other).unwrap_or(false) || other.has(self).unwrap_or(false) { return Ok(true); }

        //   -----
        //      -----
        let s = match self.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n + T::one(),
            Bound::Unbounded => T::min_(),
        };
        let e = match self.end_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n - T::one(),
            Bound::Unbounded => T::max_(),
        };

        Ok(other.contains(&s) || other.contains(&e))
    }
}



struct RangeChecker<T, U: RangeBounds<T>> {
    vec: Vec<U>,
    _phantom: core::marker::PhantomData<T>,
}



impl<T: PartialOrd + PartialEq + Ord + Eq + Add + Sub + Utils, U: RangeBounds<T> + Clone> RangeChecker<T, U> {
    pub fn new(range: U) -> Self {
        Self {vec: vec![range], _phantom: core::marker::PhantomData}
    }

    // pub fn substract(&mut self, other: &impl RangeBounds<T>) {
    //     let mut new_vec = Vec::new();
    //     for r in self.vec.iter() {
    //         match r.substract(other) {
    //             Ok((Some(r1), Some(r2))) => {
    //                 new_vec.push(r1);
    //                 new_vec.push(r2);
    //             },
    //             Ok((Some(r1), None)) => {
    //                 new_vec.push(r1);
    //             },
    //             Ok((None, Some(r2))) => {
    //                 new_vec.push(r2);
    //             },
    //             Ok((None, None)) => {},
    //             Err(()) => {
    //                 new_vec.push(r.clone());
    //             },
    //         }
    //     }
    //     self.vec = new_vec;
    // }
}

fn foo() {
    let r = 0..=8;
    let y = 0..8;

    let checker = RangeChecker::new(0..8);
    let checker = RangeChecker::new(0..=8);

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let r = 0..0;
        assert_eq!(r.is_empty(), true);

        let r = 0..=0;
        assert_eq!(r.is_empty(), false);

        let r = u8::MAX..u8::MAX;
        assert_eq!(r.is_empty(), true);

        let r = u8::MIN..u8::MIN;
        assert_eq!(r.is_empty(), true);

        let r = u8::MAX..u8::MIN;
        assert_eq!(r.is_empty(), true);

        let r = u8::MIN..u8::MAX;
        assert_eq!(r.is_empty(), false);

        assert_eq!((1..0).is_empty(), true);
        assert_eq!((1..=0).is_empty(), true);

        assert_eq!((..u8::MIN).is_empty(), true);
        assert_eq!((u8::MAX..).is_empty(), false);
    }

    #[test]
    fn contains() {
        assert_eq!((u8::MIN..=u8::MAX).has(&(..=u8::MAX)), Ok(true));
        assert_eq!((..=u8::MAX).has(&(u8::MIN..=u8::MAX)), Ok(true));
        assert_eq!((u8::MIN..=u8::MAX).has(&(..)), Ok(true));
        assert_eq!((..).has(&(u8::MIN..u8::MAX)), Ok(true));
        assert_eq!((..).has(&(u8::MIN..=u8::MAX)), Ok(true));

        assert_eq!((..).has(&(0..)), Ok(true));
        assert_eq!((..).has(&(..42)), Ok(true));
        assert_eq!((..).has(&(0..42)), Ok(true));
        assert_eq!((..).has(&(0..=42)), Ok(true));

        assert_eq!((2..42).has(&(2..42)), Ok(true));
        assert_eq!((2..42).has(&(3..42)), Ok(true));
        assert_eq!((2..42).has(&(3..41)), Ok(true));

        assert_eq!((2..42).has(&(2..=42)), Ok(false));
        assert_eq!((2..42).has(&(2..43)), Ok(false));
        assert_eq!((2..42).has(&(1..42)), Ok(false));
        assert_eq!((2..42).has(&(1..44)), Ok(false));

        assert_eq!((2..u8::MAX).has(&(2..u8::MAX)), Ok(true));
        assert_eq!((2..u8::MAX).has(&(2..=u8::MAX)), Ok(false));
        assert_eq!((2..u8::MAX).has(&(3..u8::MAX)), Ok(true));

        assert_eq!((2..u8::MAX).has(&(1..u8::MAX)), Ok(false));
        assert_eq!((2..u8::MAX).has(&(2..u8::MAX-1)), Ok(true));
        assert_eq!((2..u8::MAX).has(&(2..=u8::MAX-1)), Ok(true));

        assert_eq!((2..=u8::MAX).has(&(2..u8::MAX)), Ok(true));
        assert_eq!((2..=u8::MAX).has(&(2..=u8::MAX)), Ok(true));

        assert_eq!((2..=u8::MAX-1).has(&(2..u8::MAX)), Ok(true));
        assert_eq!((2..=u8::MAX-1).has(&(2..=u8::MAX)), Ok(false));

        assert_eq!((0..10).has(&(0..0)), Err(()));
        assert_eq!((0..0).has(&(0..10)), Err(()));
        assert_eq!((0..0).has(&(0..0)), Err(()));
    }

    #[test]
    fn equal() {
        assert_eq!((0..100).equal(&(0..=99)), true);
        assert_eq!((0u8..).equal(&(0..=u8::MAX)), true);
        assert_eq!((..).equal(&(0..=u8::MAX)), true);
        assert_eq!((..).equal(&(u8::MIN..=u8::MAX)), true);
        assert_eq!((..).equal(&(u8::MIN..u8::MAX)), false);
        assert_eq!((0..=u8::MAX).equal(&(..)), true);
    }

    #[test]
    fn sub() {
        assert_eq!((..).substract(&(u8::MIN..=u8::MAX)), Ok((None, None)));
        assert_eq!((u8::MIN..=u8::MAX).substract(&(..)), Ok((None, None)));
        assert_eq!((..).substract(&(u8::MIN..u8::MAX)), Ok((None, Some(255..=255u8))));
        assert_eq!((..=u8::MAX).substract(&(u8::MIN..u8::MAX)), Ok((None, Some(255..=255u8))));
        assert_eq!((..=u8::MAX).substract(&(..u8::MAX)), Ok((None, Some(255..=255u8))));
        assert_eq!((..=u8::MAX).substract(&(1..u8::MAX)), Ok((Some(0..=0), Some(255..=255u8))));

        assert_eq!((0..100).substract(&(30..50)), Ok((Some(0..=29), Some(50..=99))));
        assert_eq!((0..100).substract(&(30..100)), Ok((Some(0..=29), None)));
        assert_eq!((0..100).substract(&(0..50)), Ok((None, Some(50..=99))));

        assert_eq!((20..40).substract(&(30..50)), Err(()));
    }

    #[test]
    fn intersect() {
        assert_eq!((0..50).intersect(&(50..100)), Ok(false));
        assert_eq!((0..=50).intersect(&(50..100)), Ok(true));
    }

}
