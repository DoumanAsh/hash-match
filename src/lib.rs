//! Hashed match
//!
//!
//! ## Usage
//!
//! ```rust
//! use hash_match::{Matcher, Function};
//!
//! fn branch_1(_: ()) -> &'static str {
//!     "branch_1"
//! }
//! fn branch_2(_: ()) -> &'static str {
//!     "branch_2"
//! }
//! fn branch_3(_: ()) -> &'static str {
//!     "branch_3"
//! }
//! fn branch_4(_: ()) -> &'static str {
//!     "branch_4"
//! }
//! fn branch_5(_: ()) -> &'static str {
//!     "branch_5"
//! }
//!
//! fn default_branch(_: ()) -> &'static str {
//!     "default"
//! }
//!
//! const MATCHER: Matcher<5, (), &'static str> = Matcher::new([
//!     (b"branch_1", Function(branch_1)),
//!     (b"branch_2", Function(branch_2)),
//!     (b"branch_3", Function(branch_3)),
//!     (b"branch_4", Function(branch_4)),
//!     (b"branch_5", Function(branch_5)),
//! ], Function(default_branch));
//!
//! assert_eq!(MATCHER.call(b"invalid", ()), "default");
//! assert_eq!(MATCHER.call(b"branch_1", ()), "branch_1");
//! assert_eq!(MATCHER.call(b"branch_12", ()), "default");
//! assert_eq!(MATCHER.call(b"branch_2", ()), "branch_2");
//! assert_eq!(MATCHER.call(b"branch_3", ()), "branch_3");
//! assert_eq!(MATCHER.call(b"branch_4", ()), "branch_4");
//! assert_eq!(MATCHER.call(b"branch_5", ()), "branch_5");
//! ```

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use core::mem::MaybeUninit;
use core::fmt;

use xxhash_rust::{xxh3, const_xxh3};

#[repr(transparent)]
///Function pointer
pub struct Function<ARGS, R>(pub fn(ARGS) -> R);

impl<ARGS, R> Clone for Function<ARGS, R> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Function(self.0)
    }
}

impl<ARGS, R> Copy for Function<ARGS, R> {
}

impl<ARGS, R> fmt::Debug for Function<ARGS, R> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Function")
    }
}

///Simple implementation that performs logarithmic lookup of hashes values
pub struct Matcher<const N: usize, ARGS=(), R=()> {
    hashes: [u128; N],
    branches: [MaybeUninit<Function<ARGS, R>>; N],
    default: Function<ARGS, R>,
}

impl<const N: usize, ARGS, R> Matcher<N, ARGS, R> {
    ///Computes matching branches and returns matcher instance
    pub const fn new(patterns: [(&[u8], Function<ARGS, R>); N], default: Function<ARGS, R>) -> Self {
        let mut hashes = [0u128; N];
        let mut branches = [MaybeUninit::uninit(); N];
        let mut pattern_idx = 0;
        while pattern_idx < patterns.len() {
            let (key, function) = patterns[pattern_idx];

            let key_hash = const_xxh3::xxh3_128(key);

            let mut matches_idx = 0;
            while matches_idx < hashes.len() {
                let matches_key = hashes[matches_idx];
                if matches_key == 0 {
                    hashes[matches_idx] = key_hash;
                    branches[matches_idx] = MaybeUninit::new(function);
                    break;
                } else if matches_key == key_hash {
                    panic!("Collision detected");
                } else if key_hash > matches_key {
                    matches_idx += 1;
                } else {
                    //new hash has to be inserted in between
                    let mut matches_end_idx = hashes.len() - 1;
                    while matches_end_idx > matches_idx {
                        hashes[matches_end_idx] = hashes[matches_end_idx - 1];
                        branches[matches_end_idx] = branches[matches_end_idx - 1];
                        matches_end_idx -= 1;
                    }
                    hashes[matches_idx] = key_hash;
                    branches[matches_idx] = MaybeUninit::new(function);
                    break;
                }
            }

            pattern_idx += 1;
        }

        #[cfg(debug_assertions)]
        {
            pattern_idx = 1;
            while pattern_idx < hashes.len() {
                assert!(hashes[pattern_idx - 1] <= hashes[pattern_idx], "Programming error, unsorted matcher");
                pattern_idx += 1;
            }
        }

        Self {
            hashes,
            branches,
            default,
        }
    }

    #[inline]
    ///Matches `pattern`, dispatching `args` and returning result of function call of match.
    pub fn call(&self, pattern: &[u8], args: ARGS) -> R {
        let key = xxh3::xxh3_128(pattern);
        match self.hashes.binary_search(&key) {
            Ok(idx) => unsafe {
                (self.branches.get_unchecked(idx).assume_init().0)(args)
            },
            Err(_) => (self.default.0)(args)
        }
    }
}
