use crate::binaries::get_binaries;
use crate::client::SednaClient;
use crate::config::generate_sedna_config;
use crate::error::{Result, SednaError};
use std::fs;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

pub struct SednaServer {
    #[allow(dead_code)]
    data_dir: TempDir,
    #[allow(dead_code)]
    cfg_dir: TempDir,
    port: u16,
    binaries: Arc<crate::binaries::ExtractedBinaries>,
}

impl SednaServer {
    pub fn new() -> Result<Self> {
        Self::with_port(5050)
    }

    pub fn with_port(port: u16) -> Result<Self> {
        // Step 1: Extract binaries
        let binaries = get_binaries()?;

        // Step 2: Create data and config directories
        let data_dir = tempfile::tempdir().map_err(|e| {
            SednaError::ServerStartupFailed(format!("Failed to create data dir: {}", e))
        })?;

        let cfg_dir = tempfile::tempdir().map_err(|e| {
            SednaError::ServerStartupFailed(format!("Failed to create cfg dir: {}", e))
        })?;

        // Create subdirectories in data
        fs::create_dir(data_dir.path().join("data")).map_err(|e| {
            SednaError::ServerStartupFailed(format!("Failed to create data/data: {}", e))
        })?;

        fs::create_dir(data_dir.path().join("cfg")).map_err(|e| {
            SednaError::ServerStartupFailed(format!("Failed to create data/cfg: {}", e))
        })?;

        // Step 3: Generate config file
        let config_content = generate_sedna_config(data_dir.path(), port)?;
        let config_path = cfg_dir.path().join("sednaconf.xml");
        fs::write(&config_path, config_content).map_err(|e| {
            SednaError::ServerStartupFailed(format!("Failed to write config: {}", e))
        })?;

        // Step 4: Set SEDNA_INSTALL environment variable
        let sedna_install = data_dir.path().to_string_lossy().to_string();

        // Step 5: Start se_gov (must be started BEFORE creating database)
        // Use background mode so se_gov daemonizes properly
        // Set cwd to bin_dir and run as "./se_gov" so argv[0] is resolvable on macOS
        let output = Command::new("./se_gov")
            .current_dir(&binaries.bin_dir)
            .arg("-background-mode")
            .arg("on")
            .arg("-el-level")
            .arg("4")
            .env("SEDNA_INSTALL", &sedna_install)
            .env("CONFIG_FILE", &config_path)
            .output()
            .map_err(|e| {
                SednaError::ServerStartupFailed(format!("Failed to start se_gov: {}", e))
            })?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SednaError::ServerStartupFailed(format!(
                "se_gov failed to start:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        // Step 6: Wait for server to initialize
        // Give se_gov time to set up shared memory and semaphores
        thread::sleep(Duration::from_secs(2));


        // Step 7: Create a test database (needs running governor)
        // Run from bin_dir so all utilities can find each other via ./
        let output = Command::new("./se_cdb")
            .current_dir(&binaries.bin_dir)
            .arg("testdb")
            .env("SEDNA_INSTALL", &sedna_install)
            .env("CONFIG_FILE", &config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                SednaError::DatabaseCreationFailed(format!("Failed to run se_cdb: {}", e))
            })?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SednaError::DatabaseCreationFailed(format!(
                "se_cdb failed:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        // Step 8: Start the database (se_sm starts the session manager for the database)
        let output = Command::new("./se_sm")
            .current_dir(&binaries.bin_dir)
            .arg("testdb")
            .env("SEDNA_INSTALL", &sedna_install)
            .env("CONFIG_FILE", &config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                SednaError::DatabaseCreationFailed(format!("Failed to run se_sm: {}", e))
            })?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SednaError::DatabaseCreationFailed(format!(
                "se_sm failed to start database:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        // Step 9: Wait for server to be ready (check if port is listening)
        let max_attempts = 50;
        let mut connected = false;

        for _ in 0..max_attempts {
            if TcpStream::connect(format!("localhost:{}", port)).is_ok() {
                connected = true;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        if !connected {
            return Err(SednaError::ServerStartupFailed(
                "Server did not start listening on port within timeout".to_string(),
            ));
        }

        Ok(Self {
            data_dir,
            cfg_dir,
            port,
            binaries,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn connect(&self, db_name: &str, login: &str, password: &str) -> Result<SednaClient> {
        SednaClient::connect("localhost", self.port, db_name, login, password)
    }
}

impl Drop for SednaServer {
    fn drop(&mut self) {
        // Stop the Sedna server using se_stop utility
        let sedna_install = self.data_dir.path().to_string_lossy().to_string();
        let config_path = self.cfg_dir.path().join("sednaconf.xml");

        let _ = Command::new("./se_stop")
            .current_dir(&self.binaries.bin_dir)
            .env("SEDNA_INSTALL", &sedna_install)
            .env("CONFIG_FILE", config_path.to_string_lossy().as_ref())
            .output();
    }
}
