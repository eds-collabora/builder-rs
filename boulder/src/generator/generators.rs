//! Standard generator implementations.

use crate::Generator;

pub use boulder_derive::repeat as Repeat;
pub use boulder_derive::string_pattern as Pattern;

use num::One;

/// Returns the same value every time.
#[derive(Clone)]
pub struct Const<T>(pub T);

impl<T> Const<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone + 'static> Generator for Const<T> {
    type Output = T;
    fn generate(&mut self) -> Self::Output {
        self.0.clone()
    }
}

/// Each result is 1 larger than the previous.
///
/// The type `T` must implement `AddAssign + num::One + Clone`, which
/// is true for all primitive numeric types.
#[derive(Clone)]
pub struct Inc<T>(pub T);

impl<T> Generator for Inc<T>
where
    T: core::ops::AddAssign<T> + One + Clone + 'static,
{
    type Output = T;
    fn generate(&mut self) -> T {
        let res = self.0.clone();
        self.0 += T::one();
        res
    }
}

/// Yield values from an iterator, repeating them when the iterator is
/// exhausted.
pub struct Cycle<T>(::std::iter::Cycle<T>);

impl<T: Iterator + Clone> Cycle<T> {
    pub fn new(iter: T) -> Self {
        Self(iter.cycle())
    }
}

impl<S, T> Generator for Cycle<S>
where
    S: Iterator<Item = T> + Clone + 'static,
{
    type Output = T;
    fn generate(&mut self) -> T {
        self.0.next().unwrap()
    }
}

/// Convert a stream of `T` into a stream of `Option<T>` by wrapping
/// each value in `Some`.
pub struct Some<T>(pub T);

impl<T: Generator> Generator for Some<T> {
    type Output = Option<<T as Generator>::Output>;
    fn generate(&mut self) -> Self::Output {
        ::std::option::Option::Some(self.0.generate())
    }
}

impl<F, T> Generator for F
where
    F: FnMut() -> T + 'static,
{
    type Output = T;
    fn generate(&mut self) -> Self::Output {
        self()
    }
}

/// Produce collections from a pair of generators, one for the values
/// themselves, one for the size of yielded collection.
pub struct Sample<T, U, V> {
    value: T,
    count: U,
    _result_marker: core::marker::PhantomData<V>,
}

impl<T, U, V> Sample<T, U, V> {
    /// Create a new generator.
    ///
    /// The values it yields (of type `V`) will contain a number of
    /// elements of type type `T` which are determined by the sequence
    /// produced by `count`. The actual elements are generated by
    /// `value`.
    ///
    /// Example:
    /// ```rust
    /// use boulder::Generator;
    /// use boulder::gen::{Inc, Pattern, Sample};
    ///
    /// let mut g = Sample::new(
    ///     Pattern!("hello-{}", Inc(2i32)),
    ///     Inc(1usize)
    /// );
    /// let s1: Vec<_> = g.generate();
    /// assert_eq!(s1.len(), 1);
    /// assert_eq!(s1[0], "hello-2".to_string());
    /// let s2 = g.generate();
    /// assert_eq!(s2.len(), 2);
    /// assert_eq!(s2[0], "hello-3".to_string());
    /// assert_eq!(s2[1], "hello-4".to_string());
    /// ```
    pub fn new(value: T, count: U) -> Self {
        Self {
            value,
            count,
            _result_marker: Default::default(),
        }
    }
}

impl<T, U, V, X> Generator for Sample<T, U, V>
where
    T: Generator<Output = X>,
    U: Generator<Output = usize>,
    V: FromIterator<X> + 'static,
{
    type Output = V;
    fn generate(&mut self) -> Self::Output {
        super::GeneratorMutIterator {
            gen: &mut self.value,
        }
        .take(self.count.generate())
        .collect()
    }
}

/// A sequence of evenly spaced times.
pub struct Time<T: chrono::TimeZone> {
    instant: chrono::DateTime<T>,
    step: chrono::Duration,
}

impl<T: chrono::TimeZone> Time<T> {
    /// Create a new `Time` generator. Here
    /// - `start` is the first `DateTime` in the sequence.
    /// - `step` is the duration separating any two adjacent times in
    ///    the sequence.
    /// Example:
    /// ```rust
    /// use boulder::Generator;
    /// use boulder::gen::Time;
    /// use chrono::{DateTime, Duration};
    ///
    /// let mut g = Time::new(
    ///     DateTime::parse_from_rfc3339("2022-03-31T17:40:00+01:00").unwrap(),
    ///     Duration::hours(1)
    /// );
    /// let t = g.generate();
    /// assert_eq!(t, DateTime::parse_from_rfc3339("2022-03-31T17:40:00+01:00").unwrap());
    /// let t = g.generate();
    /// assert_eq!(t, DateTime::parse_from_rfc3339("2022-03-31T18:40:00+01:00").unwrap());
    /// let t = g.generate();
    /// assert_eq!(t, DateTime::parse_from_rfc3339("2022-03-31T19:40:00+01:00").unwrap());
    /// ```
    pub fn new(start: chrono::DateTime<T>, step: chrono::Duration) -> Self {
        Self {
            instant: start,
            step,
        }
    }
}

impl<T: chrono::TimeZone + 'static> Generator for Time<T> {
    type Output = chrono::DateTime<T>;
    fn generate(&mut self) -> Self::Output {
        let res = self.instant.clone();
        self.instant = self.instant.clone() + self.step;
        res
    }
}

/// A predictable sequence of subsets of a base collection.
///
/// The pattern for the yielded values is:
/// 1. vec!\[\]
/// 2. vec!\[a\]
/// 3. vec!\[b\]
/// 4. vec!\[a,b\]
/// 5. vec!\[c\]
/// 6. vec!\[a,c\]
/// 7. vec!\[b,c\]
/// 8. vec!\[a,b,c\]
/// 9. ...
///
/// where the `a` is the first element of the base collection, `b` is
/// the second, and so on.
#[derive(Clone)]
pub struct Subsets<T: Clone> {
    base: Vec<T>,
    index: usize,
}

impl<T: Clone> Subsets<T> {
    pub fn new<X: IntoIterator<Item = T>>(base: X) -> Self {
        Self {
            base: base.into_iter().collect(),
            index: 0,
        }
    }
}

impl<T: Clone + 'static> Generator for Subsets<T> {
    type Output = Vec<T>;
    fn generate(&mut self) -> Self::Output {
        let mut v = Vec::new();
        for i in 0..std::cmp::min(std::mem::size_of::<usize>() * 8, self.base.len()) {
            if self.index & (1usize << i) != 0 {
                v.push(self.base[i].clone());
            }
        }
        self.index += 1;
        v
    }
}

/// Recycle a fixed base collection.
///
/// This is a less flexible, but more concisely constructed, version
/// of `Cycle`.
#[derive(Clone)]
pub struct Repeat<T: Clone> {
    base: Vec<T>,
    index: usize,
}

impl<T: Clone> Repeat<T> {
    pub fn new<X: IntoIterator<Item = T>>(base: X) -> Self {
        Self {
            base: base.into_iter().collect(),
            index: 0,
        }
    }
}

impl<T: Clone + 'static> Generator for Repeat<T> {
    type Output = T;
    fn generate(&mut self) -> Self::Output {
        let res = self.base[self.index % self.base.len()].clone();
        self.index = (self.index + 1usize) % self.base.len();
        res
    }
}
