pub mod dataframe;
pub mod data;
pub mod grouping;
pub mod set;
pub mod mtcars;
pub mod sp500;

#[cfg(feature = "arrow")]
pub mod datafusion;

#[cfg(feature = "polars")]
pub mod polars;

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Apply a permutation to a vector in-place using cycle-following algorithm.
/// 
/// This is an O(n) time, O(n) space algorithm that follows permutation cycles
/// to rearrange elements without creating a temporary copy of the data.
/// 
/// # Arguments
/// * `data` - The vector to permute in-place
/// * `indices` - The permutation to apply, where `indices[i]` is the index
///               that element `i` should move to
/// 
/// # Panics
/// Panics if `indices` is not a valid permutation (wrong length or invalid indices)
pub fn apply_permutation_in_place<T>(data: &mut [T], indices: &[usize]) {
    assert_eq!(data.len(), indices.len(), "Permutation must have same length as data");
    
    let mut done = vec![false; data.len()];
    for cycle_start in 0..data.len() {
        if done[cycle_start] {
            continue;
        }
        let mut i = cycle_start;
        loop {
            done[i] = true;
            let next = indices[i];
            assert!(next < data.len(), "Invalid permutation index: {}", next);
            if next == cycle_start {
                break;  // Completed the cycle
            }
            data.swap(i, next);
            i = next;
        }
    }
}