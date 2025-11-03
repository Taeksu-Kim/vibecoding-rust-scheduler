use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

pub struct DaemonProcess {
    pid_file: PathBuf,
}

impl DaemonProcess {
    pub fn new() -> anyhow::Result<Self> {
        let project_dirs = ProjectDirs::from("com", "scheduler", "scheduler")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine project directory"))?;

        let data_dir = project_dirs.data_dir().to_path_buf();
        fs::create_dir_all(&data_dir)?;

        let pid_file = data_dir.join("daemon.pid");

        Ok(Self { pid_file })
    }

    pub fn is_running(&self) -> bool {
        if !self.pid_file.exists() {
            return false;
        }

        if let Ok(pid_str) = fs::read_to_string(&self.pid_file) {
            if let Ok(_pid) = pid_str.trim().parse::<u32>() {
                // Windows에서는 프로세스 존재 확인이 복잡하므로
                // 일단 PID 파일이 있으면 실행 중으로 간주
                return true;
            }
        }

        false
    }

    pub fn write_pid(&self) -> anyhow::Result<()> {
        let pid = std::process::id();
        fs::write(&self.pid_file, pid.to_string())?;
        Ok(())
    }

    pub fn remove_pid(&self) -> anyhow::Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file)?;
        }
        Ok(())
    }

    pub fn start(&self) -> anyhow::Result<()> {
        if self.is_running() {
            anyhow::bail!("Daemon is already running");
        }

        self.write_pid()?;
        log::info!("Daemon started with PID: {}", std::process::id());

        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        if !self.is_running() {
            anyhow::bail!("Daemon is not running");
        }

        self.remove_pid()?;
        log::info!("Daemon stopped");

        Ok(())
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let _ = self.remove_pid();
    }
}
