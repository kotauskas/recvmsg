use crate::{AsyncRecvMsgExt, MsgBuf, RecvResult};
use std::{mem::MaybeUninit, str::from_utf8};
use tokio::{net::UdpSocket, try_join};

#[tokio::test]
async fn v4() {
    udp(false).await
}
#[tokio::test]
async fn v6() {
    udp(true).await
}

async fn udp(v6: bool) {
    let addr = if v6 { "::1" } else { "127.0.0.1" };
    // The following two will choose different ports:
    let (mut s1, mut s2) =
        try_join!(UdpSocket::bind((addr, 0)), UdpSocket::bind((addr, 0)),).expect("bind failed");

    let getport = |sock: &UdpSocket| sock.local_addr().expect("port query failed").port();
    let (p1, p2) = dbg!((getport(&s1), getport(&s2)));

    try_join!(s1.connect((addr, p2)), s2.connect((addr, p1)),).expect("connect failed");

    let mut bufa = [MaybeUninit::new(0); 6];
    let mut buf1 = MsgBuf::from(bufa.as_mut());
    let mut buf2 = MsgBuf::from(Vec::with_capacity(16));

    let msg = "\
This message is definitely too huge for bufa, and will generally require multiple resizes unless \
the memory allocator decides to be smarter than usual and give us a huge buffer on the first try";

    let (ssz1, ssz2) =
        try_join!(s1.send(msg.as_bytes()), s2.send(msg.as_bytes())).expect("send failed");
    assert_eq!(ssz1, msg.len());
    assert_eq!(ssz2, msg.len());

    let comck = |rslt, buf: &mut MsgBuf<'_>| {
        dbg!(&*buf);
        dbg!(rslt);
        assert!(matches!(rslt, RecvResult::Spilled(sz) if sz == msg.len()));
        assert_eq!(buf.len_filled(), msg.len());
        assert_eq!(from_utf8(buf.filled_part()).expect("invalid UTF-8"), msg);
    };
    let (rslt1, rslt2) =
        try_join!(s1.recv_msg(&mut buf1), s2.recv_msg(&mut buf2)).expect("receive failed");
    comck(rslt1, &mut buf1);
    comck(rslt2, &mut buf2);
}
