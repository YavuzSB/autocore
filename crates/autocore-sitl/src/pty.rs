//! autocore-sitl :: Linux Sanal Terminal (PTY / Pseudo-Terminal) Altyapısı

use autocore_base::HalError;
use std::ffi::CStr;
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use tokio::fs::File as TokioFile;
use tracing::info;

/// Linux Pseudo-Terminal (PTY) Çifti (Master / Slave)
pub struct VirtualPtyPair {
    master_fd: RawFd,
    slave_fd: RawFd,
    slave_path: PathBuf,
}

impl VirtualPtyPair {
    /// Yeni bir Linux PTY çifti açar (`libc::openpty` ile)
    pub fn open() -> Result<Self, HalError> {
        let mut master_fd: RawFd = -1;
        let mut slave_fd: RawFd = -1;
        let mut name_buf = [0i8; 64];

        let ret = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                name_buf.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };

        if ret != 0 {
            return Err(HalError::SerialError(format!(
                "openpty çağrısı başarısız oldu: errno={}",
                std::io::Error::last_os_error()
            )));
        }

        // PTY'yi raw moda al: ICANON, ECHO vb. devre dışı bırakılır; saf ikili (binary) baytlar akar
        unsafe {
            let mut tio: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(slave_fd, &mut tio) == 0 {
                libc::cfmakeraw(&mut tio);
                libc::tcsetattr(slave_fd, libc::TCSANOW, &tio);
            }
            if libc::tcgetattr(master_fd, &mut tio) == 0 {
                libc::cfmakeraw(&mut tio);
                libc::tcsetattr(master_fd, libc::TCSANOW, &tio);
            }
        }

        let c_str = unsafe { CStr::from_ptr(name_buf.as_ptr()) };
        let slave_path = PathBuf::from(c_str.to_string_lossy().into_owned());

        info!("Sanal PTY açıldı: Slave yolu = {:?}", slave_path);

        Ok(Self {
            master_fd,
            slave_fd,
            slave_path,
        })
    }

    /// Slave tarafının Linux aygıt yolunu döndürür (Örn: `/dev/pts/3`)
    pub fn slave_path(&self) -> &Path {
        &self.slave_path
    }

    /// Master tarafını asenkron Tokio File akışı olarak döndürür
    pub fn master_tokio_file(&self) -> Result<TokioFile, HalError> {
        let dup_fd = unsafe { libc::dup(self.master_fd) };
        if dup_fd < 0 {
            return Err(HalError::SerialError("master_fd kopyalanamadı".into()));
        }
        let std_file = unsafe { File::from_raw_fd(dup_fd) };
        Ok(TokioFile::from_std(std_file))
    }

    /// Slave tarafını asenkron Tokio File akışı olarak açar
    pub fn open_slave_tokio(&self) -> Result<TokioFile, HalError> {
        let std_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.slave_path)
            .map_err(|e| HalError::SerialError(e.to_string()))?;

        Ok(TokioFile::from_std(std_file))
    }
}

impl Drop for VirtualPtyPair {
    fn drop(&mut self) {
        unsafe {
            if self.master_fd >= 0 {
                libc::close(self.master_fd);
            }
            if self.slave_fd >= 0 {
                libc::close(self.slave_fd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_virtual_pty_read_write() {
        let pty = VirtualPtyPair::open().expect("PTY açılamadı");
        assert!(pty.slave_path().exists());

        let mut master = pty.master_tokio_file().unwrap();
        let mut slave = pty.open_slave_tokio().unwrap();

        let msg = b"AUTOCORE_SITL_PTY_PING\n";
        master.write_all(msg).await.unwrap();
        master.flush().await.unwrap();

        let mut read_buf = [0u8; 64];
        let n = slave.read(&mut read_buf).await.unwrap();
        assert_eq!(&read_buf[..n], msg);
    }
}
