use std::error::Error;
pub use manyerrors_proc::{manyerrors,stash,iter_stash};


///Struct capable of containing many errors.
#[derive(Clone, PartialEq, Eq)]
pub struct Errors<T> {
    errs: Vec<T>
}

impl<T> Default for Errors<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for Errors<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.errs.into_iter()
    }
}

impl<'a,T> IntoIterator for &'a Errors<T> {
    type Item = &'a T;
    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.errs).into_iter()
    }
}

impl<A> FromIterator<A> for Errors<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<A>>().into()
    }
}

pub struct IterStash<'l,E,A: StashErrors<E>,I: Iterator<Item = A>> {
    iter: I,
    stash: &'l mut Errors<E>
}

impl<'l,E,A: StashErrors<E>,I: Iterator<Item = A>> IterStash<'l,E,A,I> {
    pub fn new(iter: impl IntoIterator<IntoIter = I>, stash: &'l mut Errors<E>) -> Self {
        IterStash {
            iter: iter.into_iter(),
            stash: stash
        }
    }
}

impl<'l,E,A: StashErrors<E>,I: Iterator<Item = A>> Iterator for IterStash<'l,E,A,I> {
    type Item = <A as StashErrors<E>>::O;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.next() {
            if let Some(r) = v.stash_errs(self.stash) {
                return Some(r);
            }
        }
        None
    }
}

pub trait LazyErrs<O,E>: Sized {
    type Iter;

    ///Take this iterator, move all the errors to the final element 
    ///so `.collect::<Result<_,Errors<_>>>()` collects all errors in the iterator.
    /// 
    /// # example
    /// 
    ///```
    /// assert_eq!(
    ///    [Ok(1),Err(1),Ok(2),Err(2),Ok(3),Err(3)].into_iter().lazy_errs().collect::<Vec<_>>(),
    ///    vec![Ok(1),Ok(2),Ok(3),Err(Errors::<i32>::from(vec![1,2,3]))]
    ///);
    ///```
    fn lazy_errs(self) -> Self::Iter;
}

impl<O,E,A: StashErrors<E, O = O>,I: Iterator<Item = A>> LazyErrs<O,E> for I {
    type Iter = LazyErrIter<O,E,A,I>;

    fn lazy_errs(self) -> Self::Iter {
        LazyErrIter {
            iter: self,
            stash: Errors::new()
        }
    }
}

pub struct LazyErrIter<O,E,A: StashErrors<E, O = O>,I: Iterator<Item = A>> {
    iter: I,
    stash: Errors<E>
}

impl<O,E,A: StashErrors<E, O = O>,I: Iterator<Item = A>> Iterator for LazyErrIter<O,E,A,I> {
    type Item = Result<O,Errors<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.next() {
            if let Some(r) = v.stash_errs(&mut self.stash) {
                return Some(Ok(r));
            }
        }
        if self.stash.errs.len() == 0 {
            None
        } else {
            Some(Err(std::mem::take(&mut self.stash)))
        }
    }
}

#[cfg(feature = "anyhow")]
pub mod anyhow {
    use crate::Errors;

    //Akin to `anyhow::Result`, but able to return multiple errors.
    pub type Result<T> = std::result::Result<T,Errors<anyhow::Error>>;
}

impl<T: std::fmt::Debug> std::fmt::Debug for Errors<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            for (id,i) in self.errs.iter().enumerate() {
                writeln!(f,"{id}: {:#?}",i)?;
            }
        } else {
            let mut is_first = true;
            for i in self.errs.iter() {
                if !is_first {
                    write!(f,", ")?;
                }
                is_first = false;
                write!(f,"{:?}",i)?;
            }
        }
        Ok(())
    }
}

impl<T> Errors<T> {
    ///Get a new empty `Errors` struct.
    pub fn new() -> Self {
        Errors { errs: Vec::new() }
    }

    pub fn add_result<O,E>(mut self, res: Result<O,E>) -> Result<O,Errors<T>>
    where Result<O,E>: StashErrors<T, O = O> {
        let v = res.stash_errs(&mut self);
        if self.errs.is_empty() && let Some(val) = v { Ok(val) } else {Err(self)}
    }

    ///Returns an iterator over all errors inside the object. `errors.iter()`
    /// is equivalent to `(&errors).into_iter()`
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        (&self).into_iter()
    }
}

///creates a new Errors struct containing just the new value.
///
///```
/// use manyerrors::err;
/// assert_eq!(err("Error"),"Error".into());
///```
pub fn err<T>(e: T) -> Errors<T> {
    Errors {
        errs: vec![e]
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Errors<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            for (id,i) in self.errs.iter().enumerate() {
                writeln!(f,"{id}: {i}")?;
            }
        } else {
            let mut is_first = true;
            for i in self.errs.iter() {
                if !is_first {
                    write!(f,", ")?;
                }
                is_first = false;
                write!(f,"{i}")?;
            }
        }
        Ok(())
    }
}

impl<T> From<T> for Errors<T> {
    fn from(value: T) -> Self {
        Errors {
            errs: vec![value]
        }
    }
}

impl<T> From<Vec<T>> for Errors<T> {
    fn from(value: Vec<T>) -> Self {
        Errors {
            errs: value
        }
    }
}

impl<T: std::fmt::Display + std::fmt::Debug> Error for Errors<T> {}

pub trait Ok<T> {
    type O;

    ///Turns an optional type into an empty result.
    ///Used with the `?` try operator to return on `None` values.
    ///
    /// # example
    ///```
    /// let o = stash!(some_result);
    /// //some logic...
    /// do_stuff(o.ok()?)
    ///```
    fn ok(self) -> Result<Self::O,Errors<T>>;
}

impl<T,O> Ok<T> for Option<O> {
    type O = O;
    fn ok(self) -> Result<Self::O,Errors<T>> {
        self.ok_or_else(|| Errors::new())
    }
}

pub trait StashErrors<E> {
    type O;
    fn stash_errs(self, stash: &mut Errors<E>) -> Option<Self::O>;
}

impl<O,E> StashErrors<E> for Result<O,E> {
    type O = O;

    fn stash_errs(self, stash: &mut Errors<E>) -> Option<Self::O> {
        match self {
            Self::Ok(o) => Some(o),
            Self::Err(e) => { stash.errs.push(e); None }
        }
    }
}

impl<O,E> StashErrors<E> for Result<O,Errors<E>> {
    type O = O;

    fn stash_errs(self, stash: &mut Errors<E>) -> Option<Self::O> {
        match self {
            Self::Ok(o) => Some(o),
            Self::Err(e) => { stash.errs.extend(e.errs.into_iter()); None }
        }
    }
}

impl<T> StashErrors<T> for T {
    type O = ();
    fn stash_errs(self, stash: &mut Errors<T>) -> Option<Self::O> {
        stash.errs.push(self);
        None
    }
}

impl<T> StashErrors<T> for Errors<T> {
    type O = ();
    fn stash_errs(self, stash: &mut Errors<T>) -> Option<Self::O> {
        for i in self.errs {
            stash.errs.push(i);
        }
        Some(())
    }
}