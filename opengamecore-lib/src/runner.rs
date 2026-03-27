use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::{Child, Command};

use crate::config::WineConfig;
use crate::dxvk;
use crate::error::{Error, Result};

/// Everything needed to launch a game.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub wine_binary: PathBuf,
    pub prefix: PathBuf,
    pub exe: PathBuf,
    pub env: HashMap<String, String>,
}

impl LaunchConfig {
    pub fn new(
        wine: &WineConfig,
        bottle_dir: &Path,
        exe_relative: &str,
        game_env: &HashMap<String, String>,
        dxvk_enabled: bool,
    ) -> Self {
        let mut env = wine.env_overrides.clone();
        if dxvk_enabled {
            env.extend(dxvk::env_overrides());
        }
        env.extend(game_env.clone());

        Self {
            wine_binary: wine.binary_path.clone(),
            prefix: bottle_dir.to_path_buf(),
            exe: bottle_dir.join(exe_relative),
            env,
        }
    }
}

/// Spawn a Wine process for the given launch config.
pub fn spawn(config: &LaunchConfig) -> Result<Child> {
    if !config.wine_binary.exists() {
        return Err(Error::WineNotFound(format!(
            "Binary not found: {}",
            config.wine_binary.display()
        )));
    }

    let child = Command::new(&config.wine_binary)
        .arg(&config.exe)
        .env("WINEPREFIX", &config.prefix)
        .envs(&config.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Process(format!("Failed to spawn Wine: {}", e)))?;

    Ok(child)
}

/// Result of running a game, including captured output.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub slug: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_secs: f64,
}

/// Spawn a Wine process and capture its output.
pub async fn run_and_capture(config: &LaunchConfig, slug: &str) -> Result<RunResult> {
    let start = std::time::Instant::now();

    let mut child = spawn(config)?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut buf = String::new();
            use tokio::io::AsyncReadExt;
            let _ = reader.read_to_string(&mut buf).await;
            buf
        } else {
            String::new()
        }
    });

    let stderr_handle = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut buf = String::new();
            use tokio::io::AsyncReadExt;
            let _ = reader.read_to_string(&mut buf).await;
            buf
        } else {
            String::new()
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| Error::Process(format!("Failed waiting for process: {}", e)))?;

    let stdout_text = stdout_handle.await.unwrap_or_default();
    let stderr_text = stderr_handle.await.unwrap_or_default();

    let duration = start.elapsed();

    Ok(RunResult {
        slug: slug.to_string(),
        exit_code: status.code(),
        stdout: stdout_text,
        stderr: stderr_text,
        duration_secs: duration.as_secs_f64(),
    })
}

/// Save a run result's logs to the app's log directory.
pub fn save_run_log(result: &RunResult) -> Result<()> {
    let log_dir = crate::paths::data_dir()?.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let log_path = log_dir.join(format!("{}-{}.log", result.slug, timestamp));

    let content = format!(
        "Game: {}\nExit code: {:?}\nDuration: {:.1}s\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}\n",
        result.slug, result.exit_code, result.duration_secs, result.stdout, result.stderr
    );

    std::fs::write(&log_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_config_merges_env() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::from([("WINEDEBUG".into(), "-all".into())]),
        };
        let game_env = HashMap::from([("DXVK_HUD".into(), "1".into())]);

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
            false,
        );

        assert_eq!(config.env.get("WINEDEBUG").unwrap(), "-all");
        assert_eq!(config.env.get("DXVK_HUD").unwrap(), "1");
        assert_eq!(
            config.exe,
            PathBuf::from("/bottles/game/drive_c/game.exe")
        );
        assert_eq!(config.prefix, PathBuf::from("/bottles/game"));
    }

    #[test]
    fn launch_config_game_env_overrides_wine_env() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::from([("KEY".into(), "wine-val".into())]),
        };
        let game_env = HashMap::from([("KEY".into(), "game-val".into())]);

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
            false,
        );

        assert_eq!(config.env.get("KEY").unwrap(), "game-val");
    }

    #[test]
    fn launch_config_dxvk_enabled_sets_dll_overrides() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::new(),
        };
        let game_env = HashMap::new();

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
            true,
        );

        let overrides = config.env.get("WINEDLLOVERRIDES").unwrap();
        assert!(overrides.contains("d3d11=n"));
        assert!(overrides.contains("dxgi=n"));
    }

    #[test]
    fn launch_config_dxvk_disabled_no_dll_overrides() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::new(),
        };
        let game_env = HashMap::new();

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
            false,
        );

        assert!(config.env.get("WINEDLLOVERRIDES").is_none());
    }

    #[tokio::test]
    async fn spawn_fails_if_binary_missing() {
        let config = LaunchConfig {
            wine_binary: PathBuf::from("/nonexistent/wine"),
            prefix: PathBuf::from("/tmp/prefix"),
            exe: PathBuf::from("/tmp/game.exe"),
            env: HashMap::new(),
        };
        assert!(spawn(&config).is_err());
    }

    #[tokio::test]
    async fn run_and_capture_returns_result_on_missing_binary() {
        let config = LaunchConfig {
            wine_binary: PathBuf::from("/nonexistent/wine"),
            prefix: PathBuf::from("/tmp/prefix"),
            exe: PathBuf::from("/tmp/game.exe"),
            env: HashMap::new(),
        };
        // Should error because binary doesn't exist
        assert!(run_and_capture(&config, "test-game").await.is_err());
    }
}
