use std::fs::OpenOptions;
use std::io::Write;

pub struct Injector {
    file: std::fs::File,
}

impl Injector {
    pub fn new(pid: i32, fd: u64) -> std::io::Result<Self> {
        let path = format!("/proc/{}/fd/{}", pid, fd);
        let file = OpenOptions::new().write(true).read(false).open(path)?;
        Ok(Injector { file })
    }

    pub fn inject(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.file.write_all(data)?;
        self.file.flush()?;
        Ok(())
    }
}
