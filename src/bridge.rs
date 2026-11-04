//! Instance-scoped local bridge. Workers handle bytes; the GUI owns all dispatch.
use crate::mcp::{self, Call, Failure, Reply};
use serde::{Deserialize, Serialize};
use std::{
    io::BufReader,
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    time::{Duration, Instant},
};
const TIMEOUT: Duration = Duration::from_secs(10);
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    address: SocketAddr,
    token: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    token: String,
    call: Call,
}
pub struct Pending {
    pub call: Call,
    pub deadline: Instant,
    response: SyncSender<Reply>,
}
impl Pending {
    pub fn reply(self, reply: Reply) {
        let _ = self.response.send(reply);
    }
}
pub struct Bridge {
    directory: tempfile::TempDir,
    requests: Receiver<Pending>,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
fn failure(code: &str, message: impl ToString) -> Failure {
    Failure {
        code: code.into(),
        message: message.to_string(),
    }
}
impl Bridge {
    pub fn start(wake: impl Fn() + Send + Sync + 'static) -> std::io::Result<Self> {
        // Current qualification is Unix: private directory permissions protect the token.
        #[cfg(not(unix))]
        return Err(std::io::Error::other(
            "Local bridge needs platform-specific private file permissions; currently qualified on Unix only.",
        ));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let directory = tempfile::Builder::new().prefix("pcbcraft-mcp-").tempdir()?;
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))?;
            let listener = TcpListener::bind("127.0.0.1:0")?;
            listener.set_nonblocking(true)?;
            let mut token = [0u8; 32];
            getrandom::fill(&mut token).map_err(|e| std::io::Error::other(e.to_string()))?;
            let token = token.iter().map(|b| format!("{b:02x}")).collect::<String>();
            let connection = Connection {
                address: listener.local_addr()?,
                token: token.clone(),
            };
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.path().join("connection.json"))?;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
            serde_json::to_writer(file, &connection).map_err(std::io::Error::other)?;
            let (tx, requests) = mpsc::sync_channel(8);
            let stop = Arc::new(AtomicBool::new(false));
            let stopped = stop.clone();
            let worker = std::thread::spawn(move || {
                while !stopped.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                            let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                            let result = receive(&mut stream, &token, &tx, &wake, &stopped);
                            let _ = mcp::write_frame(&mut stream, &result, mcp::MAX_RESPONSE);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(15))
                        }
                        Err(_) => break,
                    }
                }
            });
            Ok(Self {
                directory,
                requests,
                stop,
                worker: Some(worker),
            })
        }
    }
    pub fn path(&self) -> PathBuf {
        self.directory.path().join("connection.json")
    }
    pub fn next(&self) -> Option<Pending> {
        if self.stop.load(Ordering::Acquire) {
            return None;
        }
        self.requests.try_recv().ok()
    }
    /// Revoke this bridge immediately; callers may finish joining its IO worker off-thread.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn receive(
    stream: &mut TcpStream,
    token: &str,
    tx: &SyncSender<Pending>,
    wake: &impl Fn(),
    stop: &AtomicBool,
) -> Reply {
    let bytes = mcp::read_frame(&mut BufReader::new(stream), mcp::MAX_MESSAGE)
        .map_err(|e| failure("transport", e))?
        .ok_or_else(|| failure("transport", "Missing request"))?;
    let envelope: Envelope = serde_json::from_slice(&bytes)
        .map_err(|_| failure("invalid_input", "Invalid bridge envelope"))?;
    if envelope.token != token {
        return Err(failure("unauthorized", "Invalid instance token"));
    }
    if stop.load(Ordering::Acquire) {
        return Err(failure("disconnected", "Bridge stopped; no request queued"));
    }
    let (response, rx) = mpsc::sync_channel(1);
    let deadline = Instant::now() + TIMEOUT;
    tx.try_send(Pending {
        call: envelope.call,
        deadline,
        response,
    })
    .map_err(|_| failure("busy", "Bridge queue full or closed; no request queued"))?;
    wake();
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(reply) => return reply,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(failure("disconnected", "App session closed"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
            return Err(failure(
                "outcome_unknown",
                "No reply before disconnect/timeout. Inspect edit_status with the original request_id before retrying; execution may have completed.",
            ));
        }
    }
}
pub fn connect(path: &Path) -> std::io::Result<Connection> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(4097)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(std::io::Error::other("Invalid connection file"));
    }
    let connection: Connection = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    if !connection.address.ip().is_loopback() || connection.token.len() != 64 {
        return Err(std::io::Error::other(
            "Connection must be a loopback PCBcraft instance",
        ));
    }
    Ok(connection)
}
impl Connection {
    pub fn call(&self, call: Call) -> Reply {
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(2))
            .map_err(|e| failure("disconnected", e))?;
        stream
            .set_read_timeout(Some(TIMEOUT + Duration::from_secs(2)))
            .map_err(|e| failure("transport", e))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| failure("transport", e))?;
        mcp::write_frame(
            &mut stream,
            &Envelope {
                token: self.token.clone(),
                call,
            },
            mcp::MAX_MESSAGE,
        )
        .map_err(|e| failure("transport", e))?;
        let bytes = mcp::read_frame(&mut BufReader::new(stream), mcp::MAX_RESPONSE)
            .map_err(|e| {
                failure(
                    "outcome_unknown",
                    format!("{e}; inspect edit_status before retrying mutations"),
                )
            })?
            .ok_or_else(|| {
                failure(
                    "outcome_unknown",
                    "App disconnected; inspect edit_status before retrying mutations",
                )
            })?;
        serde_json::from_slice(&bytes).map_err(|e| failure("transport", e))?
    }
}
#[cfg(test)]
mod tests;
