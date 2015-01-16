
//! Helper methods for working with arrays.

/// Implemented by arrays of different lengths.
pub trait Array<T> {
    /// Creates array from a function of each component index.
    fn from_fn<F>(f: F) -> Self where F: FnMut(usize) -> T;

    /// Creates an array from an iterator.
    /// Will fail if the iterator does not contain enough elements.
    fn from_iter<I: Iterator<Item=T>>(mut iter: I) -> Self where Self: Sized {
        Array::from_fn(|&mut:_| { iter.next().unwrap() })
    }
}

impl<T> Array<T> for [T; 2] {
    fn from_fn<F>(mut f: F) -> [T; 2] where F: FnMut(usize) -> T {
        [f(0), f(1)]
    }
}

/// An array with 2 components.
pub trait Array2<T> {
    /// Converts array into another type,
    /// by executing a function for each component.
    fn map<U, F>(self, f: F) -> [U; 2] where F: Fn(T) -> U;
    /// Returns the `x` component.
    fn x(self) -> T;
    /// Returns the `y` component.
    fn y(self) -> T;
}

impl<T: Copy> Array2<T> for [T; 2] {
    fn map<U, F>(self, f: F) -> [U; 2] where F: Fn(T) -> U {
        let [a, b] = self;
        [f(a), f(b)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
}

impl<T> Array<T> for [T; 3] {
    fn from_fn<F>(mut f: F) -> [T; 3] where F: FnMut(usize) -> T {
        [f(0), f(1), f(2)]
    }
}

/// An array with 3 components.
pub trait Array3<T> {
    /// Converts array into another type,
    /// by executing a function for each component.
    fn map<U, F>(self, f: F) -> [U; 3] where F: Fn(T) -> U;
    /// Returns the `x` component.
    fn x(self) -> T;
    /// Returns the `y` component.
    fn y(self) -> T;
    /// Returns the `z` component.
    fn z(self) -> T;
}

impl<T: Copy> Array3<T> for [T; 3] {
    fn map<U, F>(self, f: F) -> [U; 3] where F: Fn(T) -> U {
        let [a, b, c] = self;
        [f(a), f(b), f(c)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
    fn z(self) -> T { self[2] }
}

impl<T> Array<T> for [T; 4] {
    fn from_fn<F>(mut f: F) -> [T; 4] where F: FnMut(usize) -> T {
        [f(0), f(1), f(2), f(3)]
    }
}

/// An array with 4 components.
pub trait Array4<T> {
    /// Converts array into another type,
    /// by executing a function for each component.
    fn map<U, F>(self, f: F) -> [U; 4] where F: Fn(T) -> U;
    /// Returns the `x` component.
    fn x(self) -> T;
    /// Returns the `y` component.
    fn y(self) -> T;
    /// Returns the `z` component.
    fn z(self) -> T;
    /// Returns the `w` component.
    fn w(self) -> T;
}

impl<T: Copy> Array4<T> for [T; 4] {
    fn map<U, F>(self, f: F) -> [U; 4] where F: Fn(T) -> U {
        let [a, b, c, d] = self;
        [f(a), f(b), f(c), f(d)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
    fn z(self) -> T { self[2] }
    fn w(self) -> T { self[3] }
}

impl<T> Array<T> for [T; 16] {
    fn from_fn<F>(mut f: F) -> [T; 16] where F: FnMut(usize) -> T {
        [
            f(0), f(1), f(2), f(3),
            f(4), f(5), f(6), f(7),
            f(8), f(9), f(10),f(11),
            f(12),f(13),f(14),f(15)
        ]
    }
}
