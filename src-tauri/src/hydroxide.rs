use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

/// Global handle to the hydroxide child process we spawned (if any).
/// Only set when *we* started it — never if it was already running.
static HYDROXIDE_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

const HYDROXIDE_BIN: &str = "/Users/map/go/bin/hydroxide";
const HYDROXIDE_HOST: &str = "127.0.0.1";
const HYDROXIDE_IMAP_PORT: u16 = 1143;

/// Check if hydroxide is already listening on the IMAP port.
fn is_hydroxide_running() -> bool {
    std::net::TcpStream::connect_timeout(
        &format!("{}:{}", HYDROXIDE_HOST, HYDROXIDE_IMAP_PORT)
            .parse()
            .unwrap(),
        Duration::from_millis(500),
    )
    .is_ok()
}

/// Ensure hydroxide is running. Spawns it if needed and waits until the
/// IMAP port becomes available (up to ~15 seconds).
pub fn ensure_running() {
    if is_hydroxide_running() {
        log::info!("hydroxide is already running on port {}", HYDROXIDE_IMAP_PORT);
        return;
    }

    log::info!("hydroxide not detected, starting '{} serve'...", HYDROXIDE_BIN);

    let child = Command::new(HYDROXIDE_BIN)
        .arg("serve")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(child) => {
            log::info!("hydroxide spawned (pid {})", child.id());
            *HYDROXIDE_PROCESS.lock().unwrap() = Some(child);

            // Wait for IMAP port to become available
            for i in 0..30 {
                if is_hydroxide_running() {
                    log::info!("hydroxide ready after ~{}ms", i * 500);
                    return;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
            log::warn!("hydroxide started but IMAP port not ready after 15s");
        }
        Err(e) => {
            log::error!("Failed to start hydroxide: {}", e);
        }
    }
}

/// Kill the hydroxide process if we started it. Called on app exit.
pub fn stop_if_we_started() {
    let mut guard = HYDROXIDE_PROCESS.lock().unwrap();
    if let Some(ref mut child) = *guard {
        log::info!("Stopping hydroxide (pid {})...", child.id());
        let _ = child.kill();
        let _ = child.wait();
        log::info!("hydroxide stopped");
    }
    *guard = None;
}
