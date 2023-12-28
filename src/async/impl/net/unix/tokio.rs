use crate::sync::r#impl::net::unix as syncimpl;
use tokio::net;

impl_atrm!(for [net::UdpSocket, net::UnixDatagram], with syncimpl::recv_trunc);
impl_arm!(for [net::UdpSocket, net::UnixDatagram], with syncimpl::recv_msg);
#[cfg(any(target_os = "linux", target_os = "android"))]
impl_atrmwfs!(for [net::UdpSocket, net::UnixDatagram], with syncimpl::recv_trunc_with_full_size);
