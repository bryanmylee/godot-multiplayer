pub mod config;
pub mod identity;
pub mod queue;
pub mod websocket;

use std::collections::BinaryHeap;

trait BinaryHeapExt<T> {
    fn remove<F>(&mut self, f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool;
}

impl<T> BinaryHeapExt<T> for BinaryHeap<T>
where
    T: Eq + Ord + Clone,
{
    fn remove<F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut removed: Option<T> = None;
        self.retain(|e| {
            let matches = f(e);
            if matches {
                removed = Some(e.clone());
            }
            !matches
        });
        removed
    }
}
