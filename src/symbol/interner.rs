use string_interner::StringInterner;

use std::fmt;
use std::sync::Mutex;

// By using `usize` here, we guarantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
// TODO: usize would be a u32 on 32-bit platforms. Is it at all possible there will ever
//       be more than 2^32 symbols?
// NOTE: Ordering of InternedString is entirely based on the opaque details of the global
//       `STRING_INTERNER` and is therefore not consistant between runs, and should be
//       considered to be essentially random and non-deterministic.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
struct InternedString(usize);

// This macro comes from the static_assertions crate.
assert_eq_size!(InternedString, usize);

lazy_static! {
    static ref STRING_INTERNER: Mutex<StringInterner<usize>> =
        Mutex::new(StringInterner::new());
}

fn acquire_lock() -> ::std::sync::MutexGuard<'static, StringInterner<usize>> {
    // Mutex::lock DOES block.
    // When running tests, many `#[test] fn ...`'s are run in parallel, sharing
    // the global STRING_INTERNER. We expect therefore to sometimes have to wait
    // to acquire the lock. Benchmarks should not be run in parallel for this reason.
    STRING_INTERNER
        .lock()
        .expect("STRING_INTERNER Mutex was poisoned!")
}

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO?: Drop this lock before the write!. write! will block, and in pathological
        // cases, could cause other threads attempting to create/format symbols to block
        // longer than necessary in acquire_lock() (assuming the try_lock().expect(...) is
        // replaced with lock()).
        //
        // Average case performance might be better if, after acquiring the lock and
        // resolve to a &str, we create a String and immediately drop the lock, *then* do
        // the write!. That prevents holding the lock while doing a write!.
        let lock = acquire_lock();
        // TODO: Replace this with resolve_unchecked once I'm confident that's correct.
        let s: &str = match lock.resolve(self.0) {
            Some(s) => s,
            None => panic!(
                "Failed to resolve InternedString from global \
                 STRING_INTERNER. InternedString id: {}",
                self.0
            ),
        };
        write!(f, "{}", s)
    }
}

impl PartialEq<str> for InternedString {
    fn eq(&self, other: &str) -> bool {
        let mut lock = acquire_lock();
        // TODO: We don't actually need to intern other to do this comparison, maybe we
        //       shouldn't? This means that doing a test like `my_symbol == "Global`A"`
        //       would add `my_symbol` to STRING_INTERNER. That seems harmless, but I'm
        //       not sure.
        let other_sym: usize = lock.get_or_intern(other);
        self.0 == other_sym
    }
}

impl<S: Into<String> + AsRef<str>> From<S> for InternedString {
    fn from(s: S) -> InternedString {
        let mut lock = acquire_lock();
        let value: usize = lock.get_or_intern(s);
        InternedString(value)
    }
}
