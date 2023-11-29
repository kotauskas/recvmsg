use super::MsgBuf;
use core::{mem::MaybeUninit, num::NonZeroUsize};

#[test]
fn ensure_capacity() {
    let mut bufbak = [0; 1];
    let mut buf = MsgBuf::from(bufbak.as_mut());
    buf.ensure_capacity(2).unwrap();
    assert!(buf.capacity() >= 2);
    buf.ensure_capacity(9).unwrap();
    assert!(buf.capacity() >= 9);

    buf = MsgBuf::from(alloc::vec::Vec::with_capacity(15));
    buf.ensure_capacity(32).unwrap();
    assert!(buf.capacity() >= 32);
    buf.ensure_capacity(33).unwrap();
    assert!(buf.capacity() >= 33);

    buf.quota = NonZeroUsize::new(32);
    assert!(buf.ensure_capacity(64).is_err());
    buf.quota = NonZeroUsize::new(64);
    buf.ensure_capacity(64).unwrap();
    assert!(buf.capacity() >= 64);
}

#[test]
fn extend() {
    let mut bufbak = [MaybeUninit::new(0); 10];
    let mut buf = MsgBuf::from(bufbak.as_mut());
    buf.extend_from_slice(&[1; 10]).unwrap();
    assert_eq!(buf.len_filled(), 10);
}
