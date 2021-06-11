use smoltcp::socket::{
    SocketHandle as InnerSocketHandle, SocketSet as InnerSocketSet, TcpSocket, TcpSocketBuffer,
    UdpPacketMetadata, UdpSocket, UdpSocketBuffer,
};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone, Copy)]
pub struct BufferSize {
    pub tcp_rx_size: usize,
    pub tcp_tx_size: usize,
    pub udp_rx_size: usize,
    pub udp_tx_size: usize,
    pub udp_meta_size: usize,
}

impl Default for BufferSize {
    fn default() -> Self {
        BufferSize {
            tcp_rx_size: 8192,
            tcp_tx_size: 8192,
            udp_rx_size: 8192,
            udp_tx_size: 8192,
            udp_meta_size: 32,
        }
    }
}

pub struct SocketAlloctor {
    buffer_size: BufferSize,
    set: Arc<Mutex<InnerSocketSet<'static>>>,
}

impl SocketAlloctor {
    pub fn new(buffer_size: BufferSize) -> SocketAlloctor {
        SocketAlloctor {
            buffer_size,
            set: Arc::new(Mutex::new(InnerSocketSet::new(vec![]))),
        }
    }
    pub fn lock(&self) -> MutexGuard<InnerSocketSet<'static>> {
        self.set.lock().unwrap()
    }
    pub fn new_tcp_socket(&self) -> SocketHandle {
        let mut set = self.set.lock().unwrap();
        let handle = set.add(self.alloc_tcp_socket());
        SocketHandle::new(handle, self.set.clone())
    }
    pub fn new_udp_socket(&self) -> SocketHandle {
        let mut set = self.set.lock().unwrap();
        let handle = set.add(self.alloc_udp_socket());
        SocketHandle::new(handle, self.set.clone())
    }
    fn alloc_tcp_socket(&self) -> TcpSocket<'static> {
        let rx_buffer = TcpSocketBuffer::new(vec![0; self.buffer_size.tcp_rx_size]);
        let tx_buffer = TcpSocketBuffer::new(vec![0; self.buffer_size.tcp_tx_size]);
        let tcp = TcpSocket::new(rx_buffer, tx_buffer);

        tcp
    }
    fn alloc_udp_socket(&self) -> UdpSocket<'static> {
        let rx_buffer = UdpSocketBuffer::new(
            vec![UdpPacketMetadata::EMPTY; self.buffer_size.udp_meta_size],
            vec![0; self.buffer_size.udp_rx_size],
        );
        let tx_buffer = UdpSocketBuffer::new(
            vec![UdpPacketMetadata::EMPTY; self.buffer_size.udp_meta_size],
            vec![0; self.buffer_size.udp_tx_size],
        );
        let udp = UdpSocket::new(rx_buffer, tx_buffer);

        udp
    }
}

pub struct SocketHandle(InnerSocketHandle, Arc<Mutex<InnerSocketSet<'static>>>);

impl SocketHandle {
    fn new(inner: InnerSocketHandle, set: Arc<Mutex<InnerSocketSet<'static>>>) -> SocketHandle {
        SocketHandle(inner, set)
    }
}

impl Drop for SocketHandle {
    fn drop(&mut self) {
        let mut set = self.1.lock().unwrap();
        set.remove(self.0);
    }
}

impl Deref for SocketHandle {
    type Target = InnerSocketHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SocketHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
