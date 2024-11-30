//! Utility functions to work with elements.

use std::ops::{Add, Mul, Sub};

/// An exponential decay interpolation function.
///
/// This function interpolates between points `a` and `b` given the provided decay factor `k` and
/// delta time `dt` in seconds.
///
/// `k` is proportional to the "inverse half life". It determines how fast the
/// interpolation will converge to the target value (b). The higher the value, the faster
/// the interpolation will converge. That value should be multiplied by a delta time in seconds
/// to get framerate-independent interpolation.
pub fn exp_decay<T>(a: T, b: T, k: f64) -> T
where
    T: Copy + Sub<Output = T> + Mul<f64, Output = T> + Add<Output = T>,
{
    b + (a - b) * (-k).exp()
}
