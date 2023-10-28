// socketcan/src/lib.rs
//
// The main lib file for the Rust SocketCAN library.
//
// This file is part of the Rust 'socketcan-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.

//! SocketCAN address type.

use libc::{sa_family_t, sockaddr, sockaddr_can, sockaddr_storage, socklen_t};
use nix::net::if_::if_nametoindex;
use socket2::SockAddr;
use std::{fmt, io, mem, os::raw::c_int};

pub use libc::{AF_CAN, CAN_RAW, PF_CAN};

/// CAN socket address.
///
/// This is the address for use with CAN sockets. It is simply an addres to
/// the SocketCAN host interface. It can be created by looking up the name
/// of the interface, like "can0", "vcan0", etc, or an interface index can
/// be specified directly, if known. An index of zero can be used to read
/// frames from all interfaces.
///
/// This is based on, and compatible with, the `sockaddr_can` struct from
/// libc.
/// [ref](https://docs.rs/libc/latest/libc/struct.sockaddr_can.html)
#[derive(Clone, Copy)]
pub struct CanAddr(sockaddr_can);

impl CanAddr {
    /// Creates a new CAN socket address for the specified interface by index.
    /// An index of zero can be used to read from all interfaces.
    pub fn new(ifindex: u32) -> Self {
        let mut addr = Self::default();
        addr.0.can_ifindex = ifindex as c_int;
        addr
    }

    /// Try to create an address from an interface name.
    pub fn from_iface(ifname: &str) -> io::Result<Self> {
        let ifindex = if_nametoindex(ifname)?;
        Ok(Self::new(ifindex))
    }

    /// Gets the address of the structure as a `sockaddr_can` pointer.
    pub fn as_ptr(&self) -> *const sockaddr_can {
        &self.0
    }

    /// Gets the address of the structure as a `sockaddr` pointer.
    pub fn as_sockaddr_ptr(&self) -> *const sockaddr {
        self.as_ptr().cast()
    }

    /// Gets the size of the address structure.
    pub fn len() -> usize {
        mem::size_of::<sockaddr_can>()
    }

    /// Gets the underlying address as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        crate::as_bytes(&self.0)
    }

    /// Converts the address into a `sockaddr_storage` type.
    /// This is a generic socket address container with enough space to hold
    /// any address type in the system.
    pub fn into_storage(self) -> (sockaddr_storage, socklen_t) {
        let can_addr = self.as_bytes();
        let len = can_addr.len();

        let mut storage: sockaddr_storage = unsafe { mem::zeroed() };
        let sock_addr = crate::as_bytes_mut(&mut storage);

        sock_addr[..len].copy_from_slice(can_addr);
        (storage, len as socklen_t)
    }

    /// Converts the address into a `socket2::SockAddr`
    pub fn into_sock_addr(self) -> SockAddr {
        SockAddr::from(self)
    }
}

impl Default for CanAddr {
    fn default() -> Self {
        let mut addr: sockaddr_can = unsafe { mem::zeroed() };
        addr.can_family = AF_CAN as sa_family_t;
        Self(addr)
    }
}

impl fmt::Debug for CanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CanAddr {{ can_family: {}, can_ifindex: {} }}",
            self.0.can_family, self.0.can_ifindex
        )
    }
}

impl From<sockaddr_can> for CanAddr {
    fn from(addr: sockaddr_can) -> Self {
        Self(addr)
    }
}

impl From<CanAddr> for SockAddr {
    fn from(addr: CanAddr) -> Self {
        let (storage, len) = addr.into_storage();
        unsafe { SockAddr::new(storage, len) }
    }
}

impl AsRef<sockaddr_can> for CanAddr {
    fn as_ref(&self) -> &sockaddr_can {
        &self.0
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::as_bytes;

    const IDX: u32 = 42;

    #[test]
    fn test_addr() {
        let _addr = CanAddr::new(IDX);

        assert_eq!(mem::size_of::<sockaddr_can>(), CanAddr::len());
    }

    #[test]
    fn test_addr_to_sock_addr() {
        let addr = CanAddr::new(IDX);

        let (sock_addr, len) = addr.clone().into_storage();

        assert_eq!(CanAddr::len() as socklen_t, len);
        assert_eq!(as_bytes(&addr), &as_bytes(&sock_addr)[0..len as usize]);
    }
}
