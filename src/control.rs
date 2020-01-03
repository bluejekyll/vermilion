// Copyright 2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::os::unix::io::RawFd;

use nix::sys::socket::{socketpair, AddressFamily, SockFlag, SockProtocol, SockType};
use nix::unistd::close;

use crate::pipe::{End, Read, Write};

#[derive(Debug)]
pub struct CtlEnd<E: End> {
    raw_fd: RawFd,
    ghost: PhantomData<E>,
}

impl<E: End> CtlEnd<E> {
    pub fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self {
            raw_fd,
            ghost: PhantomData,
        }
    }

    pub fn raw_fd(&self) -> RawFd {
        self.raw_fd
    }

    /// Forget the fd so that drop is not called after being associated to STDIN or similar
    pub fn forget(&mut self) {
        self.raw_fd = -1;
    }

    pub fn close(&mut self) -> nix::Result<()> {
        if self.raw_fd < 0 {
            return Ok(());
        }

        close(self.raw_fd)
    }
}

// TODO: requires forgetting self when STDIN or SRDOUT are attached to it...
impl<E: End> Drop for CtlEnd<E> {
    fn drop(&mut self) {
        match self.raw_fd {
            // don't implicitly close any of the std io
            0..=2 => return,
            // don't close -1, NULL
            i if i < 0 => return,
            _ => (),
        }

        println!("closing fd: {} ({})", self.raw_fd, E::display());

        // TODO: need the stdoutger...
        close(self.raw_fd)
            .map_err(|e| println!("error closing file handle: {}", self.raw_fd))
            .ok();
    }
}

pub struct Control {
    read: CtlEnd<Read>,
    write: CtlEnd<Write>,
}

impl Control {
    /// Creates a new pipe, if possible,
    ///
    /// This should be converted to the specific end desired, i.e. read() will close write() implicitly.
    ///   it's expected that this is created before forking, and then used after forking.
    pub fn new() -> nix::Result<Self> {
        let (read, write) = socketpair(
            AddressFamily::Unix,
            SockType::Stream,
            None,
            SockFlag::empty(),
        )?;

        println!("created socketpair, read: {} write: {}", read, write);

        Ok(Self {
            read: CtlEnd::from_raw_fd(read),
            write: CtlEnd::from_raw_fd(write),
        })
    }

    pub fn take_writer(self) -> CtlEnd<Write> {
        let Control { mut read, write } = self;
        write
    }

    pub fn take_reader(self) -> CtlEnd<Read> {
        let Control { read, mut write } = self;
        read
    }
}
