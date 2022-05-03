use std::cell::UnsafeCell;
use std::os::unix::io::{AsRawFd, RawFd};
use termios::Termios;
use tokio::fs::File;
use tokio::io;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncWriteExt, ErrorKind};

pub struct Session {
    tty: UnsafeCell<File>,
    pub poller: AsyncFd<RawFd>,
    cooked: Termios,
    raw: Termios,
}

impl Session {
    #[inline]
    pub fn new(tty: File) -> io::Result<Self> {
        let fd = tty.as_raw_fd();
        let poller = AsyncFd::new(fd)?;
        let cooked = Termios::from_fd(fd)?;
        let raw = {
            let mut raw = cooked.clone();

            termios::cfmakeraw(&mut raw);

            raw.c_lflag &= !(termios::ICANON
                | termios::ECHO
                | termios::ECHOE
                | termios::ECHOK
                | termios::ECHONL);

            raw.c_cc[termios::VMIN] = 0;
            raw.c_cc[termios::VTIME] = 0;

            raw
        };

        let tty = UnsafeCell::new(tty);

        Ok(Self {
            tty,
            poller,
            cooked,
            raw,
        })
    }

    #[inline]
    pub fn set_cooked(&self) -> io::Result<()> {
        termios::tcsetattr(self.as_raw_fd(), termios::TCSANOW, &self.cooked)?;

        Ok(())
    }

    #[inline]
    pub fn set_raw(&self) -> io::Result<()> {
        termios::tcsetattr(self.as_raw_fd(), termios::TCSANOW, &self.raw)?;

        Ok(())
    }

    #[inline]
    pub fn set_nonblocking(&self) -> io::Result<()> {
        unsafe {
            let flags = libc::fcntl(self.as_raw_fd(), libc::F_GETFL);

            libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        Ok(())
    }

    #[inline]
    pub fn set_blocking(&self) -> io::Result<()> {
        unsafe {
            let flags = libc::fcntl(self.as_raw_fd(), libc::F_GETFL);

            libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags & !libc::O_NONBLOCK);
        }

        Ok(())
    }

    #[inline]
    pub fn tty(&self) -> &mut File {
        unsafe { &mut *self.tty.get() }
    }

    #[inline]
    pub async fn wait_for_user(&self) -> io::Result<Vec<u8>> {
        let mut sink = Vec::new();

        loop {
            let mut guard = self.poller.readable().await?;
            let result = io::copy(self.tty(), &mut sink).await;

            match result {
                Ok(0) => {
                    guard.clear_ready();
                    continue;
                }
                Ok(_read) => {
                    guard.retain_ready();
                    break;
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    guard.clear_ready();
                    continue;
                }
                Err(_error) => {
                    guard.retain_ready();
                    break;
                }
            }
        }

        Ok(sink)
    }

    #[inline]
    pub async fn write_all(&self, buffer: &[u8]) -> io::Result<()> {
        loop {
            let result = self.tty().write_all(buffer).await;

            match result {
                Ok(_ok) => break,
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    #[inline]
    pub async fn write_str_all(&self, string: &str) -> io::Result<()> {
        self.write_all(string.as_bytes()).await?;

        Ok(())
    }
}

impl AsRawFd for Session {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        unsafe { (&mut *self.tty.get()).as_raw_fd() }
    }
}

// in case of something fatal, attempt to restore normalcy
impl Drop for Session {
    #[inline]
    fn drop(&mut self) {
        let _ = self.set_blocking();
        let _ = self.set_cooked();
    }
}
