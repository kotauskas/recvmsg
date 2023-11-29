use super::*;
use alloc::boxed::Box;
use core::ops::DerefMut;

impl<T: TruncatingRecvMsg + ?Sized, P: DerefMut<Target = T> + Unpin> TruncatingRecvMsg for Pin<P> {
    type Error = T::Error;
    forward_trait_methods! {
        pin_fn poll_recv_trunc(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<Option<bool>, Self::Error>>;
        pin_fn poll_discard_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    }
}
impl<T: TruncatingRecvMsg + Unpin + ?Sized> TruncatingRecvMsg for &mut T {
    type Error = T::Error;
    forward_trait_methods! {
        deref_fn poll_recv_trunc(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<Option<bool>, Self::Error>>;
        deref_fn poll_discard_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    }
}
impl<T: TruncatingRecvMsg + Unpin + ?Sized> TruncatingRecvMsg for Box<T> {
    type Error = T::Error;
    forward_trait_methods! {
        deref_fn poll_recv_trunc(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<Option<bool>, Self::Error>>;
        deref_fn poll_discard_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    }
}

impl<T: TruncatingRecvMsgWithFullSize + ?Sized, P: DerefMut<Target = T> + Unpin> TruncatingRecvMsgWithFullSize
    for Pin<P> // I hate Rustfmt
{
    forward_trait_methods! {
        pin_fn poll_recv_trunc_with_full_size(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<TryRecvResult, Self::Error>>;
    }
}
impl<T: TruncatingRecvMsgWithFullSize + Unpin + ?Sized> TruncatingRecvMsgWithFullSize for &mut T {
    forward_trait_methods! {
        deref_fn poll_recv_trunc_with_full_size(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<TryRecvResult, Self::Error>>;
    }
}
impl<T: TruncatingRecvMsgWithFullSize + Unpin + ?Sized> TruncatingRecvMsgWithFullSize for Box<T> {
    forward_trait_methods! {
        deref_fn poll_recv_trunc_with_full_size(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<TryRecvResult, Self::Error>>;
    }
}

impl<T: RecvMsg + ?Sized, P: DerefMut<Target = T> + Unpin> RecvMsg for Pin<P> {
    type Error = T::Error;
    forward_trait_methods! {
        pin_fn poll_recv(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<RecvResult, Self::Error>>;
    }
}
impl<T: RecvMsg + Unpin + ?Sized> RecvMsg for &mut T {
    type Error = T::Error;
    forward_trait_methods! {
        deref_fn poll_recv(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<RecvResult, Self::Error>>;
    }
}
impl<T: RecvMsg + Unpin + ?Sized> RecvMsg for Box<T> {
    type Error = T::Error;
    forward_trait_methods! {
        deref_fn poll_recv(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut MsgBuf<'_>,
        ) -> Poll<Result<RecvResult, Self::Error>>;
    }
}
