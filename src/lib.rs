#![feature(default_type_params, tuple_indexing)]
#![experimental]

use std::cell::UnsafeCell;
use std::kinds::marker::NoSync;
use std::mem::{replace, transmute};
use std::rand::{task_rng, Rng};
use std::num::Int;
use std::iter::AdditiveIterator;
use std::fmt;
use std::default::Default;
use std::hash::Hash;

/// A box that contains many values, but collapses into one when opened (read from) for the first
/// time.
///
/// Example
/// =======
///
/// ```rust
/// # use schroedinger_box::SchroedingerBox;
/// let cat_is_alive = SchroedingerBox::new(vec![true, false]);
/// // Here the cat is both dead and alive, but when we observe it...
/// let state = *cat_is_alive;
/// // ...it collapses into one of the possible states with equal probability.
/// assert_eq!(state, *cat_is_alive);
/// ```
// This should be called `SchrödingerBox`, but until type aliases can have static methods called on
// them (rust-lang/rust#11047) I’ll take pity on those barbarians who can’t type umlauts easily.
pub struct SchroedingerBox<Cat> {
    _inner: UnsafeCell<Vec<(u64, Cat)>>,
    _nosync: NoSync,
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
            _inner: UnsafeCell::new(states),
            _nosync: NoSync,
        }
    }

    /// This function is unsafe because it does lots of unsafe stuff that’s probably able to cause
    /// bad things to happen.
    unsafe fn collapse(&self) {
        // Using `UnsafeCell` is quite messy, so I hope I’ve got this bit right.
        let vec = &mut *self._inner.get();
        if vec.len() == 1 {
            return
        }
        let mut idx = {
            let len = vec.iter().map(|&(f, _)| f).sum();
            task_rng().gen_range(0, len)
        } + 1; // For some reason, we need to add 1 to idx

        let v = replace(vec, vec![]);
        let (_, val) =
            v.into_iter().skip_while(|&(f, _)| {
                idx = idx.saturating_sub(f);
                idx != 0
            }).next().unwrap();
        *vec = vec![(1, val)];
    }

    /// Moves the value inside a `SchroedingerBox` out, consuming the box and collapsing any
    /// superposition into a definite state if needed.
    pub fn into_inner(self) -> Cat {
        unsafe { self.collapse(); }
        let vec = unsafe { &mut *self._inner.get() };
        let v = replace(&mut *vec, vec![]);
        debug_assert_eq!(v.len(), 1);
        v.into_iter().next().unwrap().1
    }
}

impl<Cat> Deref<Cat> for SchroedingerBox<Cat> {
    /// Obtains a reference to the value inside a `SchroedingerBox`, collapsing any superposition
    /// into a definite state if needed.
    fn deref(&self) -> &Cat {
        unsafe {
            self.collapse();
            transmute::<&Cat, &Cat>(&(*self._inner.get())[0].1)
        }
    }
}

impl<Cat> DerefMut<Cat> for SchroedingerBox<Cat> {
    /// Obtains a mutable reference to the value inside a `SchroedingerBox`, collapsing any
    /// superposition into a definite state if needed.
    fn deref_mut(&mut self) -> &mut Cat {
        unsafe {
            self.collapse();
            transmute::<&mut Cat, &mut Cat>(&mut (*self._inner.get())[0].1)
        }
    }
}

impl<Cat> fmt::Show for SchroedingerBox<Cat>
        where Cat: fmt::Show {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<Cat> PartialEq for SchroedingerBox<Cat>
        where Cat: PartialEq {
    fn eq(&self, other: &SchroedingerBox<Cat>) -> bool {
        **self == **other
    }
}

impl<Cat> Eq for SchroedingerBox<Cat> where Cat: Eq {}

impl<Cat> PartialOrd for SchroedingerBox<Cat>
        where Cat: PartialOrd {
    fn partial_cmp(&self, other: &SchroedingerBox<Cat>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<Cat> Ord for SchroedingerBox<Cat> where Cat: Ord {
    fn cmp(&self, other: &SchroedingerBox<Cat>) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<Cat> Default for SchroedingerBox<Cat>
        where Cat: Default {
    fn default() -> SchroedingerBox<Cat> {
        SchroedingerBox::new(vec![Default::default()])
    }
}

impl<Cat> Clone for SchroedingerBox<Cat>
        where Cat: Clone {
    /// Clones a `SchroedingerBox`.
    ///
    /// This collapses any superposition into a single state.
    fn clone(&self) -> SchroedingerBox<Cat> {
        SchroedingerBox::new(vec![(**self).clone()])
    }
}

impl<Cat, S> Hash<S> for SchroedingerBox<Cat>
        where Cat: Hash<S> {
    fn hash(&self, state: &mut S) {
        (**self).hash(state)
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
            // There’s a million to one chance, but it might not work
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
        for _ in range(0u8, 100) {
            assert_eq!(*foo, val);
        }
    }

    #[test]
    fn test_into_inner() {
        let foo = SchroedingerBox::new(vec![1i, 2, 3]);
        let val = *foo;
        let own = foo.into_inner();
        assert_eq!(own, val);
    }
}
