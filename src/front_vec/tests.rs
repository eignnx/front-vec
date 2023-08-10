use crate::FrontVec;

#[test]
fn create_push_pop_drop() {
    let mut v = FrontVec::new();
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 0);

    v.push_front(5);
    v.push_front(4);
    v.push_front(1000);
    v.pop_front();
    assert_eq!(v.len(), 2);

    v.push_front(3);
    v.push_front(2);
    v.push_front(1);

    assert_eq!(v.len(), 5);
    assert_eq!(v.capacity(), 8);
    assert_eq!(v.as_ref(), &[1, 2, 3, 4, 5]);
}

#[test]
fn new_drop() {
    let v = FrontVec::<String>::new();
    drop(v);
}

#[test]
fn random_access() {
    #![allow(clippy::get_first)]

    let mut v = FrontVec::<usize>::new();
    v.push_front(5);
    v.push_front(4);
    v.push_front(3);
    v.push_front(2);
    v.push_front(1);
    v.push_front(0);

    assert_eq!(v.get(100), None);
    assert_eq!(v.get(5), Some(&5));
    assert_eq!(v.get(4), Some(&4));
    assert_eq!(v.get(3), Some(&3));
    assert_eq!(v.get(2), Some(&2));
    assert_eq!(v.get(1), Some(&1));
    assert_eq!(v.get(0), Some(&0));
}

#[test]
fn slicing() {
    let v = FrontVec::from(&[0, 1, 2, 3, 4, 5]);
    assert_eq!(&v[1..4], &[1, 2, 3]);
}

#[test]
fn index_mut() {
    let mut v = FrontVec::from(&[0, 1, 2, 3, 4, 5]);
    assert_eq!(v[1], 1);
    v[1] = 111;
    assert_eq!(v[1], 111);
}
