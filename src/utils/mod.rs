pub mod dataframe;
pub mod data;
pub mod set;
pub mod faithful;
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

pub struct MultiZipIterator<I>
where
    I: Iterator,
{
    iterators: Vec<I>,
}

impl<I> MultiZipIterator<I>
where
    I: Iterator,
{
    pub fn new(iterators: Vec<I>) -> Self {
        Self { iterators }
    }
}

/// Trait to enable `.multizip()` on vectors of iterators
pub trait IntoMultiZip<I: Iterator> {
    fn zip(self) -> MultiZipIterator<I>;
}

impl<I: Iterator> IntoMultiZip<I> for Vec<I> {
    fn zip(self) -> MultiZipIterator<I> {
        MultiZipIterator::new(self)
    }
}

impl<I> Iterator for MultiZipIterator<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut items = Vec::with_capacity(self.iterators.len());
        for iter in &mut self.iterators {
            if let Some(item) = iter.next() {
                items.push(item);
            } else {
                return None; // One of the iterators is exhausted
            }
        }
        Some(items)
    }
}

/// Iterator adapter that groups consecutive elements based on a comparison function
pub struct GroupBy<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item, &I::Item) -> std::cmp::Ordering,
{
    iter: I,
    cmp: F,
    current_group: Option<Vec<I::Item>>,
}

impl<I, F> GroupBy<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item, &I::Item) -> std::cmp::Ordering,
{
    fn new(iter: I, cmp: F) -> Self {
        Self {
            iter,
            cmp,
            current_group: None,
        }
    }
}

impl<I, F> Iterator for GroupBy<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: FnMut(&I::Item, &I::Item) -> std::cmp::Ordering,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a partial group from last time, start with it
        let mut group = self.current_group.take().or_else(|| {
            // Otherwise, try to get the first element
            self.iter.next().map(|item| vec![item])
        })?;

        // Collect consecutive items that compare equal to the last item in group
        while let Some(item) = self.iter.next() {
            if (self.cmp)(group.last().unwrap(), &item) == std::cmp::Ordering::Equal {
                group.push(item);
            } else {
                // Found an item that doesn't match - save it for next group
                self.current_group = Some(vec![item]);
                return Some(group);
            }
        }

        // Iterator exhausted - return the final group if non-empty
        if group.is_empty() {
            None
        } else {
            Some(group)
        }
    }
}

/// Extension trait to add `group_by` to iterators
pub trait GroupByExt: Iterator + Sized {
    /// Groups consecutive elements that compare equal according to the provided function.
    ///
    /// The comparison function should return `true` if two elements belong to the same group.
    ///
    /// # Example
    ///
    /// ```
    /// use gogplot::utils::GroupByExt;
    ///
    /// let data = vec![1, 1, 2, 2, 2, 3, 1, 1];
    /// let groups: Vec<Vec<i32>> = data.into_iter()
    ///     .group_by(|a, b| a.cmp(b))
    ///     .collect();
    ///
    /// assert_eq!(groups, vec![
    ///     vec![1, 1],
    ///     vec![2, 2, 2],
    ///     vec![3],
    ///     vec![1, 1],
    /// ]);
    /// ```
    fn group_by<F>(self, cmp: F) -> GroupBy<Self, F>
    where
        F: FnMut(&Self::Item, &Self::Item) -> std::cmp::Ordering,
    {
        GroupBy::new(self, cmp)
    }
}

impl<I: Iterator> GroupByExt for I {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multizip_basic() {
        let iter1 = vec![1, 2, 3].into_iter();
        let iter2 = vec![4, 5, 6].into_iter();
        let iter3 = vec![7, 8, 9].into_iter();

        let mut multizip = MultiZipIterator::new(vec![iter1, iter2, iter3]);

        assert_eq!(multizip.next(), Some(vec![1, 4, 7]));
        assert_eq!(multizip.next(), Some(vec![2, 5, 8]));
        assert_eq!(multizip.next(), Some(vec![3, 6, 9]));
        assert_eq!(multizip.next(), None);
    }

    #[test]
    fn test_multizip_with_trait() {
        let iter1 = vec![1, 2, 3].into_iter();
        let iter2 = vec![4, 5, 6].into_iter();
        let iter3 = vec![7, 8, 9].into_iter();

        let mut multizip = vec![iter1, iter2, iter3].zip();

        assert_eq!(multizip.next(), Some(vec![1, 4, 7]));
        assert_eq!(multizip.next(), Some(vec![2, 5, 8]));
        assert_eq!(multizip.next(), Some(vec![3, 6, 9]));
        assert_eq!(multizip.next(), None);
    }

    #[test]
    fn test_multizip_single_iterator() {
        let iter = vec![1, 2, 3].into_iter();
        let mut multizip = MultiZipIterator::new(vec![iter]);

        assert_eq!(multizip.next(), Some(vec![1]));
        assert_eq!(multizip.next(), Some(vec![2]));
        assert_eq!(multizip.next(), Some(vec![3]));
        assert_eq!(multizip.next(), None);
    }

    #[test]
    fn test_multizip_empty_iterator() {
        let iter: Vec<i32> = vec![];
        let multizip = MultiZipIterator::new(vec![iter.into_iter()]);

        assert_eq!(multizip.count(), 0);
    }

    #[test]
    fn test_multizip_no_iterators() {
        let multizip: MultiZipIterator<std::vec::IntoIter<i32>> = MultiZipIterator::new(vec![]);
        
        // With no iterators, should produce infinite empty vecs
        // But the first next() will try to create an empty vec and return Some(vec![])
        // Actually, with zero iterators, the for loop won't run, so it returns Some(vec![])
        let expected: Vec<Vec<i32>> = vec![vec![], vec![], vec![]];
        assert_eq!(multizip.take(3).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_multizip_different_lengths_stops_at_shortest() {
        let iter1 = vec![1, 2, 3, 4, 5].into_iter();
        let iter2 = vec![6, 7].into_iter();  // Shortest
        let iter3 = vec![8, 9, 10].into_iter();

        let mut multizip = MultiZipIterator::new(vec![iter1, iter2, iter3]);

        assert_eq!(multizip.next(), Some(vec![1, 6, 8]));
        assert_eq!(multizip.next(), Some(vec![2, 7, 9]));
        assert_eq!(multizip.next(), None);  // Stops when iter2 exhausted
    }

    #[test]
    fn test_multizip_strings() {
        let iter1 = vec!["a", "b", "c"].into_iter();
        let iter2 = vec!["x", "y", "z"].into_iter();

        let mut multizip = MultiZipIterator::new(vec![iter1, iter2]);

        assert_eq!(multizip.next(), Some(vec!["a", "x"]));
        assert_eq!(multizip.next(), Some(vec!["b", "y"]));
        assert_eq!(multizip.next(), Some(vec!["c", "z"]));
        assert_eq!(multizip.next(), None);
    }

    #[test]
    fn test_multizip_with_collect() {
        let iter1 = vec![1, 2, 3].into_iter();
        let iter2 = vec![4, 5, 6].into_iter();

        let multizip = MultiZipIterator::new(vec![iter1, iter2]);
        let result: Vec<Vec<i32>> = multizip.collect();

        assert_eq!(result, vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
    }

    #[test]
    fn test_multizip_many_iterators() {
        let iter1 = vec![1].into_iter();
        let iter2 = vec![2].into_iter();
        let iter3 = vec![3].into_iter();
        let iter4 = vec![4].into_iter();
        let iter5 = vec![5].into_iter();

        let mut multizip = MultiZipIterator::new(vec![iter1, iter2, iter3, iter4, iter5]);

        assert_eq!(multizip.next(), Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(multizip.next(), None);
    }

    #[test]
    fn test_apply_permutation_identity() {
        let mut data = vec![1, 2, 3, 4, 5];
        let indices = vec![0, 1, 2, 3, 4];
        apply_permutation_in_place(&mut data, &indices);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_apply_permutation_reverse() {
        let mut data = vec![1, 2, 3, 4, 5];
        let indices = vec![4, 3, 2, 1, 0];
        apply_permutation_in_place(&mut data, &indices);
        assert_eq!(data, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_apply_permutation_cycle() {
        let mut data = vec![1, 2, 3, 4];
        let indices = vec![1, 2, 3, 0];  // Rotate left
        apply_permutation_in_place(&mut data, &indices);
        assert_eq!(data, vec![2, 3, 4, 1]);
    }

    #[test]
    fn test_apply_permutation_swap() {
        let mut data = vec!['a', 'b', 'c', 'd'];
        let indices = vec![1, 0, 3, 2];  // Swap pairs
        apply_permutation_in_place(&mut data, &indices);
        assert_eq!(data, vec!['b', 'a', 'd', 'c']);
    }

    #[test]
    #[should_panic(expected = "Permutation must have same length as data")]
    fn test_apply_permutation_wrong_length() {
        let mut data = vec![1, 2, 3];
        let indices = vec![0, 1];  // Too short
        apply_permutation_in_place(&mut data, &indices);
    }

    #[test]
    #[should_panic(expected = "Invalid permutation index")]
    fn test_apply_permutation_invalid_index() {
        let mut data = vec![1, 2, 3];
        let indices = vec![0, 1, 5];  // 5 is out of bounds
        apply_permutation_in_place(&mut data, &indices);
    }

    #[test]
    fn test_group_by_basic() {
        let data = vec![1, 1, 2, 2, 2, 3, 1, 1];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| a.cmp(b))
            .collect();

        assert_eq!(groups, vec![
            vec![1, 1],
            vec![2, 2, 2],
            vec![3],
            vec![1, 1],
        ]);
    }

    #[test]
    fn test_group_by_single_element() {
        let data = vec![42];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| a.cmp(b))
            .collect();

        assert_eq!(groups, vec![vec![42]]);
    }

    #[test]
    fn test_group_by_empty() {
        let data: Vec<i32> = vec![];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| a.cmp(b))
            .collect();

        assert_eq!(groups, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_group_by_all_equal() {
        let data = vec![5, 5, 5, 5];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| a.cmp(b))
            .collect();

        assert_eq!(groups, vec![vec![5, 5, 5, 5]]);
    }

    #[test]
    fn test_group_by_all_different() {
        let data = vec![1, 2, 3, 4, 5];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| a.cmp(b))
            .collect();

        assert_eq!(groups, vec![
            vec![1],
            vec![2],
            vec![3],
            vec![4],
            vec![5],
        ]);
    }

    #[test]
    fn test_group_by_strings() {
        let data = vec!["apple", "apricot", "banana", "berry", "cherry"];
        let groups: Vec<Vec<&str>> = data.into_iter()
            .group_by(|a, b| a.chars().next().cmp(&b.chars().next()))  // Group by first letter
            .collect();

        assert_eq!(groups, vec![
            vec!["apple", "apricot"],
            vec!["banana", "berry"],
            vec!["cherry"],
        ]);
    }

    #[test]
    fn test_group_by_custom_predicate() {
        // Group numbers by same parity (odd/even)
        let data = vec![1, 3, 5, 2, 4, 6, 7, 9];
        let groups: Vec<Vec<i32>> = data.into_iter()
            .group_by(|a, b| (a % 2).cmp(&(b % 2)))
            .collect();

        assert_eq!(groups, vec![
            vec![1, 3, 5],
            vec![2, 4, 6],
            vec![7, 9],
        ]);
    }

    #[test]
    fn test_group_by_with_take() {
        let data = vec![1, 1, 2, 2, 3, 3, 4, 4];
        let mut grouped = data.into_iter().group_by(|a, b| a.cmp(b));

        assert_eq!(grouped.next(), Some(vec![1, 1]));
        assert_eq!(grouped.next(), Some(vec![2, 2]));
        // Take only first two groups
        drop(grouped);
    }
}