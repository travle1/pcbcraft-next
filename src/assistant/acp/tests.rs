use super::*;
use crate::assistant::{Context, Conversation, Event, Status, Worker};
use crate::session::Session;

fn run(mode: &str, record: Option<&std::path::Path>) -> Result<(Conversation, Worker)> {
    let session = Session::new(Document::empty("Original ACP fixture")?)?;
    let mut conversation = Conversation::default();
    let request = conversation.begin(
        "Explain this sheet",
        Context {
            snapshot: session.snapshot(),
            sheet: session.project().root,
            selection: json!({}),
        },
    )?;
    let mut arguments = vec![
        format!("{}/tests/fixtures/acp_peer.py", env!("CARGO_MANIFEST_DIR")),
        mode.into(),
    ];
    if let Some(record) = record {
        arguments.push(record.to_string_lossy().into());
    }
    let backend = Acp {
        config: Config {
            executable: "/usr/bin/python3".into(),
            arguments,
        },
        adapter: "/usr/bin/python3".into(),
        connection: "/tmp/fixture-unused-connection".into(),
    };
    let worker = Worker::start(Box::new(backend), request, || {})?;
    Ok((conversation, worker))
}
fn settle(conversation: &mut Conversation, worker: &Worker) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while conversation.active().is_some() && Instant::now() < deadline {
        if let Some(event) = worker.next() {
            conversation.receive(worker.turn, event);
        } else {
            std::thread::yield_now();
        }
    }
    assert!(
        conversation.active().is_none(),
        "Scripted peer did not finish"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn scripted_acp_peer_negotiates_streams_and_checks_permissions() -> Result<()> {
    let (mut conversation, worker) = run("reply", None)?;
    settle(&mut conversation, &worker);
    let message = conversation.messages().last().ok_or("Missing message")?;
    assert_eq!(message.text, "I can inspect this sheet.");
    assert_eq!(message.status, Status::Complete);
    Ok(())
}
#[cfg(target_os = "linux")]
#[test]
fn scripted_acp_peer_errors_and_protocol_limits_keep_partial_text() -> Result<()> {
    for mode in ["error", "wrong-version", "oversized"] {
        let (mut conversation, worker) = run(mode, None)?;
        settle(&mut conversation, &worker);
        let message = conversation.messages().last().ok_or("Missing message")?;
        assert!(matches!(message.status, Status::Failed(_)), "{mode}");
        if mode == "error" {
            assert_eq!(message.text, "I can inspect ");
        }
    }
    Ok(())
}
#[cfg(target_os = "linux")]
#[test]
fn stop_sends_acp_cancel_and_late_chunks_cannot_resume_turn() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let record = directory.path().join("peer.json");
    let (mut conversation, worker) = run("hang", Some(&record))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while !record.exists() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(record.exists());
    conversation.stop();
    worker.cancel();
    while Instant::now() < deadline {
        if std::fs::read_to_string(&record).is_ok_and(|s| s.contains("cancelled")) {
            break;
        }
        std::thread::yield_now();
    }
    assert!(std::fs::read_to_string(record)?.contains("cancelled"));
    assert!(!conversation.receive(worker.turn, Event::Text("late".into())));
    assert_eq!(
        conversation.messages().last().map(|m| &m.status),
        Some(&Status::Stopped)
    );
    while !worker.finished() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        worker.finished(),
        "Cancelled ACP worker must close its direct process"
    );
    Ok(())
}
