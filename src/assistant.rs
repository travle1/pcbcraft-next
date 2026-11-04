//! Bounded conversation state and a nonblocking backend seam. Edits belong to Session.
use crate::{authoring::Snapshot, document::*, mcp};
use serde::Serialize;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender, TrySendError},
};
use std::time::{Duration, Instant};

pub mod acp;

pub const MAX_PROMPT: usize = 16 * 1024;
pub const MAX_REPLY: usize = 64 * 1024;
pub const MAX_HISTORY: usize = 128 * 1024;
pub const MAX_MESSAGES: usize = 24;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Streaming,
    Complete,
    Stopped,
    Failed(String),
}
#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub role: Role,
    pub text: String,
    pub status: Status,
}
#[derive(Clone, Debug, Serialize)]
pub struct Context {
    pub snapshot: Snapshot,
    pub sheet: SheetOccurrenceId,
    pub selection: Value,
}
#[derive(Clone, Debug, Serialize)]
pub struct Request {
    pub turn: u64,
    pub context: Context,
    pub messages: Vec<Message>,
}

#[derive(Default)]
pub struct Conversation {
    messages: Vec<Message>,
    serial: u64,
    active: Option<(u64, Context)>,
    activity: String,
}
impl Conversation {
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    pub fn active(&self) -> Option<(u64, &Context)> {
        self.active.as_ref().map(|(id, context)| (*id, context))
    }
    pub fn activity(&self) -> &str {
        &self.activity
    }
    pub fn begin(&mut self, prompt: &str, context: Context) -> Result<Request> {
        if self.active.is_some() {
            return Err("Stop the current response before sending another request.".into());
        }
        let prompt = prompt.trim();
        if prompt.is_empty() || prompt.len() > MAX_PROMPT {
            return Err("A request must contain 1–16384 bytes of text.".into());
        }
        if serde_json::to_vec(&context.selection)
            .map_err(|e| Error::Invalid(e.to_string()))?
            .len()
            > MAX_PROMPT
        {
            return Err("Selection context exceeds this conversation's limit.".into());
        }
        let turn = self
            .serial
            .checked_add(1)
            .ok_or("Conversation turn limit reached.")?;
        while self.messages.len() + 2 > MAX_MESSAGES
            || self.messages.iter().map(|m| m.text.len()).sum::<usize>() + prompt.len()
                > MAX_HISTORY - MAX_REPLY
        {
            if self.messages.len() < 2 {
                break;
            }
            self.messages.drain(..2);
        }
        self.serial = turn;
        self.messages.push(Message {
            role: Role::User,
            text: prompt.into(),
            status: Status::Complete,
        });
        let request = Request {
            turn,
            context: context.clone(),
            messages: self.messages.clone(),
        };
        self.messages.push(Message {
            role: Role::Assistant,
            text: String::new(),
            status: Status::Streaming,
        });
        self.active = Some((turn, context));
        self.activity = "Connecting…".into();
        Ok(request)
    }
    /// A cancelled/older worker cannot append text or finish a later turn.
    pub fn receive(&mut self, turn: u64, event: Event) -> bool {
        if self.active.as_ref().map(|(id, _)| *id) != Some(turn) {
            return false;
        }
        let Some(message) = self.messages.last_mut() else {
            return false;
        };
        match event {
            Event::Text(text) => {
                if message.text.len() + text.len() > MAX_REPLY {
                    message.status = Status::Failed("Response exceeded the 64 KiB limit.".into());
                    self.active = None;
                } else {
                    message.text.push_str(&text);
                    self.activity = "Responding…".into();
                }
            }
            Event::Activity(text) => self.activity = text.chars().take(160).collect(),
            Event::Finished => {
                message.status = Status::Complete;
                self.active = None;
            }
            Event::Failed(text) => {
                message.status = Status::Failed(text.chars().take(1000).collect());
                self.active = None;
            }
        }
        true
    }
    pub fn stop(&mut self) {
        if self.active.take().is_some()
            && let Some(message) = self.messages.last_mut()
        {
            message.status = Status::Stopped;
        }
        self.activity.clear();
    }
    pub fn invalidate(&mut self, snapshot: &Snapshot) -> bool {
        if let Some((turn, context)) = &self.active
            && context.snapshot != *snapshot
        {
            let turn = *turn;
            self.receive(turn, Event::Failed("The project changed during this response. Send a new request using the current design.".into()));
            return true;
        }
        false
    }
    pub fn clear(&mut self) {
        self.stop();
        self.messages.clear();
        // Keep serial monotonic: even a late event after Clear cannot match a new turn.
    }
}

pub enum Event {
    Text(String),
    Activity(String),
    Finished,
    Failed(String),
}

/// Implementations run on a worker thread and never own the Electronics session.
pub trait Backend: Send + 'static {
    fn run(self: Box<Self>, request: Request, stream: &Stream) -> Result<()>;
}
pub struct Stream {
    events: SyncSender<Event>,
    cancelled: Arc<AtomicBool>,
    wake: Arc<dyn Fn() + Send + Sync>,
}
impl Stream {
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    pub fn text(&self, text: String) -> Result<()> {
        if text.len() > MAX_REPLY {
            return Err("Response chunk exceeded the 64 KiB limit.".into());
        }
        self.emit(Event::Text(text))
    }
    pub fn activity(&self, text: impl Into<String>) -> Result<()> {
        self.emit(Event::Activity(text.into().chars().take(160).collect()))
    }
    fn emit(&self, mut event: Event) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if self.cancelled() {
                return Err("Response stopped.".into());
            }
            match self.events.try_send(event) {
                Ok(()) => {
                    (self.wake)();
                    return Ok(());
                }
                Err(TrySendError::Full(returned)) if Instant::now() < deadline => {
                    event = returned;
                    std::thread::sleep(Duration::from_millis(5));
                }
                _ => return Err("Conversation closed or stopped receiving responses.".into()),
            }
        }
    }
}
pub struct Worker {
    pub turn: u64,
    events: Receiver<Event>,
    cancelled: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    pub fn start(
        backend: Box<dyn Backend>,
        request: Request,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self> {
        let (tx, events) = mpsc::sync_channel(16);
        let cancelled = Arc::new(AtomicBool::new(false));
        let stream = Stream {
            events: tx,
            cancelled: cancelled.clone(),
            wake: Arc::new(wake),
        };
        let turn = request.turn;
        let thread = std::thread::Builder::new()
            .name("pcbcraft-assistant".into())
            .spawn(move || {
                let result = backend.run(request, &stream);
                if !stream.cancelled() {
                    let _ = stream.emit(match result {
                        Ok(()) => Event::Finished,
                        Err(error) => Event::Failed(error.to_string()),
                    });
                }
            })?;
        Ok(Self {
            turn,
            events,
            cancelled,
            thread: Some(thread),
        })
    }
    pub fn next(&self) -> Option<Event> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(mpsc::TryRecvError::Disconnected) => Some(Event::Failed(
                "The connection closed before the response finished.".into(),
            )),
            Err(mpsc::TryRecvError::Empty) => None,
        }
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub fn finished(&self) -> bool {
        self.thread
            .as_ref()
            .is_none_or(|thread| thread.is_finished())
    }
    /// Used at application shutdown, never in an editor frame. Backends must honor cancellation.
    pub fn shutdown(mut self) {
        self.cancel();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !self.finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if self.finished()
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
        if self.finished()
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
    }
}

/// Enforce a turn's original snapshot and review-only permissions for every tool request.
pub fn dispatch(
    session: &mut crate::session::Session,
    context: &Context,
    call: mcp::Call,
) -> mcp::Reply {
    if session.snapshot() != context.snapshot {
        return Err(mcp::Failure {
            code: "conflict".into(),
            message: "The project changed; send a new request from the current design.".into(),
        });
    }
    if call.name == "apply_edit" || call.name == "discard_edit" {
        return Err(mcp::Failure {
            code: "review_required".into(),
            message: "Use Apply or Discard in Assistant to decide on a proposal.".into(),
        });
    }
    mcp::dispatch(
        session,
        &mcp::Context {
            active_sheet: context.sheet,
            selection: context.selection.clone(),
            allow_apply: false,
        },
        call,
    )
}

#[cfg(test)]
mod tests;
