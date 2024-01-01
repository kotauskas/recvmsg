use super::MsgBuf;
use core::mem::MaybeUninit;

#[test]
fn ensure_capacity() {
    let mut bufbak = [0; 1];
    let mut buf = MsgBuf::from(bufbak.as_mut());
    buf.ensure_capacity(2).unwrap();
    assert!(buf.capacity() >= 2);
    buf.ensure_capacity(309).unwrap();
    assert!(buf.capacity() >= 309);

    buf = MsgBuf::from(alloc::vec::Vec::with_capacity(305));
    buf.ensure_capacity(512).unwrap();
    assert!(buf.capacity() >= 512);
    buf.ensure_capacity(1025).unwrap();
    assert!(buf.capacity() >= 1025);

    buf.quota = Some(256);
    assert!(buf.ensure_capacity(4096).is_err());
    assert!(buf.capacity() >= 1025);
    buf.quota = Some(4096);
    buf.ensure_capacity(4096).unwrap();
    assert!(buf.capacity() >= 4096);

    buf = MsgBuf::from(bufbak.as_mut());
    buf.quota = Some(0);
    assert!(buf.ensure_capacity(1).is_ok());
    assert!(buf.ensure_capacity(2).is_err());
}

#[test]
fn extend() {
    let mut bufbak = [MaybeUninit::new(0); 10];
    let mut buf = MsgBuf::from(bufbak.as_mut());
    buf.extend_from_slice(&[1; 10]).unwrap();
    assert_eq!(buf.len_filled(), 10);
}
