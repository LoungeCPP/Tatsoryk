//! Various useful reusable mathematical functions.

/// Calculate the distance between two points on a plane.
///
/// # Examples
///
/// ```
/// let distance = distance_between(0, 0, 1, 1);
/// assert_eq!((distance * 10.0).round() / 10.0, 1.4);
/// ```
pub fn distance_between(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x0 - x1;
    let dy = y0 - y1;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use self::super::distance_between;

    #[test]
    fn distance_between_produces_expected_results() {
        let distance = distance_between(0.0, 0.0, 1.0, 1.0);
        assert_eq!((distance * 100000.0).round() / 100000.0, 1.41421);
    }
}
