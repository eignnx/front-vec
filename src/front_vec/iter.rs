use super::front_vec::FrontVec;

pub struct IntoIter<T> {
    v: Option<FrontVec<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.v {
            Some(v) if v.len() == 1 => {
                // SAFETY: We just checked that it's non-empty, so unwrap is ok.
                let mut v = unsafe { self.v.take().unwrap_unchecked() };
                v.pop_front()
            }
            Some(v) if !v.is_empty() => v.pop_front(),
            _ => return None,
        }
    }
}

#[test]
fn iterator() {
    let v = FrontVec::from(vec![1, 2, 3, 4, 5]);
    let mut it = IntoIter { v: Some(v) };
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.next(), None);
}
