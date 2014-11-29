#![experimental]

use std::cell::UnsafeCell;
use std::kinds::marker::NoSync;
use std::mem::{replace, transmute};
use std::ptr::read;
use std::rand::{task_rng, Rng};
use std::num::Int;
use std::iter::AdditiveIterator;

/// A box that contains one of many values, but collapses into one when opened (read from) for
/// the first time.
///
/// Example
/// =======
///
/// ```rust
/// # use schroedinger_box::SchroedingerBox;
/// let cat_is_alive = SchroedingerBox::new(vec![true, false]);
/// // Here the cat is both dead and alive, but when we observe it...
/// let state = *cat_is_alive;
/// // ...it collapses into one state.
/// assert_eq!(state, *cat_is_alive);
/// ```
// This should be called `SchrödingerBox`, but until type aliases can have static methods called on
// them (rust-lang/rust#11047) I’ll take pity on those barbarians who can’t type umlauts easily.
pub struct SchroedingerBox<Cat> {
    _inner: UnsafeCell<SchroedInner<Cat>>,
    _nosync: NoSync,
}

/// The inner contents of a `SchroedingerBox`.
enum SchroedInner<Cat> {
    /// This represents a set of states, each with a proability.
    Superposition(Vec<(u64, Cat)>),
    /// This represents a single collapsed state.
    Collapsed(Cat),
    /// We use this state in `SchroedingerBox::collapse` as a temporary state so we can gain
    /// ownership of the list of superpositions.
    Empty,
}

impl<Cat> SchroedingerBox<Cat> {
    /// Creates a new `SchroedingerBox` from a set of states.
    ///
    /// When the box is first opened, the contents’ superposition will collapse into one of the
    /// given states with equal probability.
    pub fn new(states: Vec<Cat>) -> SchroedingerBox<Cat> {
        SchroedingerBox::from_probabilities(states.into_iter().map(|x| (1, x)).collect())
    }

    /// Creates a new `SchroedingerBox` from a set of states, each with a probability.
    ///
    /// When the box is first opened, the contents’ superposition will collapse into one of the
    /// given states with each state’s probability determined by the probabilities given.
    ///
    /// The probablity for a state is represented by a ratio of an integer to the total sum of the
    /// probabilities; e.g., a set of states and probabilities `[(1, true), (5, false)]` would be
    /// `false` five sixths of the time and `true` one sixth of the time.
    // Here we *could* choose the `Collapsed` state instantly, avoiding all the trouble with
    // `UnsafeCell` and so on. But that would be boring and against the point, so we make sure that
    // the state collapses only on the first observation.
    pub fn from_probabilities(states: Vec<(u64, Cat)>) -> SchroedingerBox<Cat> {
        SchroedingerBox {
            _inner: UnsafeCell::new(SchroedInner::Superposition(states)),
            _nosync: NoSync,
        }
    }

    /// This function is unsafe because it does lots of unsafe stuff that’s probably able to cause
    /// bad things to happen.
    unsafe fn collapse(&self) {
        // Using `UnsafeCell` is quite messy, so I hope I’ve got this bit right.
        let mut borrow = &mut *self._inner.get();
        let mut idx = {
            let v = match *borrow.deref_mut() {
                SchroedInner::Superposition(ref mut v) => {
                    v
                },
                SchroedInner::Collapsed(_) => return,
                SchroedInner::Empty => unreachable!(),
            };
            let len = v.iter().map(|&(f, _)| f).sum();
            task_rng().gen_range(0, len)
        };
        let v = replace(&mut *borrow, SchroedInner::Empty);
        let (_, val) = match v {
            SchroedInner::Superposition(v) => {
                // For some reason, we need this here
                idx += 1;
                v.into_iter().skip_while(|&(f, _)| {
                    idx = idx.saturating_sub(f);
                    idx != 0
                }).next().unwrap()
            },
            _ => unreachable!(),
        };
        *borrow = SchroedInner::Collapsed(val);
    }

    /// Moves the value inside a `SchroedingerBox` out, consuming the box and collapsing any
    /// superposition into a definite state if needed.
    pub fn into_inner(self) -> Cat {
        unsafe { self.collapse(); }
        match unsafe { read(self._inner.get() as *const _) } {
            SchroedInner::Collapsed(v) => v,
            _ => unreachable!(),
        }
    }
}

impl<Cat> Deref<Cat> for SchroedingerBox<Cat> {
    /// Obtains a reference to the value inside a `SchroedingerBox`, collapsing any superposition
    /// into a definite state if needed.
    fn deref(&self) -> &Cat {
        unsafe { self.collapse(); }
        match unsafe { &*self._inner.get() } {
            &SchroedInner::Collapsed(ref v) => unsafe { transmute::<&Cat, &Cat>(v) },
            _ => unreachable!(),
        }
    }
}

impl<Cat> DerefMut<Cat> for SchroedingerBox<Cat> {
    /// Obtains a mutable reference to the value inside a `SchroedingerBox`, collapsing any
    /// superposition into a definite state if needed.
    fn deref_mut(&mut self) -> &mut Cat {
        unsafe { self.collapse(); }
        match unsafe { &mut *self._inner.get() } {
            &SchroedInner::Collapsed(ref mut v) => unsafe { transmute::<&mut Cat, &mut Cat>(v) },
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SchroedingerBox;

    #[test]
    fn whats_in_the_box() {
        // This is basically imposible to test, but we try anyway.
        // I think I’m beginning to understand how quantum physicists feel. :(

        // Set up a `SchroedingerBox` with one very unlikely value
        let foo = SchroedingerBox::from_probabilities(
            vec![(100000, 1i), (500000, 2), (499999, 3), (1, 4)]);
        let val = *foo;
        match val {
            1 | 2 | 3 => {},
            // There’s a million to one chance, but it might not work!
            4 => {
                panic!("an unlikely event occurred; this is probably a bug, \
                        but there’s a chance it isn’t");
            },
            // If this happens, our code is really broken
            _ => panic!(),
        }
    }

    #[test]
    fn collapsing_state_does_not_change() {
        let foo = SchroedingerBox::new(vec![1i, 2, 3]);
        let val = *foo;
        assert_eq!(*foo, val);
        assert_eq!(*foo, val);
        assert_eq!(*foo, val);
    }

    #[test]
    fn test_into_inner() {
        let foo = SchroedingerBox::new(vec![1i, 2, 3]);
        let val = *foo;
        let own = foo.into_inner();
        assert_eq!(own, val);
    }
}
