use super::*;
use crate::{
    authoring::Apply,
    persistence,
    session::{Command, Session},
};
use serde_json::json;

fn session() -> Result<Session> {
    Session::new(Document::empty("Original Assistant fixture")?)
}
fn context(session: &Session) -> Context {
    Context {
        snapshot: session.snapshot(),
        sheet: session.project().root,
        selection: json!({"symbols":[],"wires":[],"notes":[]}),
    }
}

#[test]
fn conversation_stream_stop_clear_and_late_events_are_isolated() -> Result<()> {
    let session = session()?;
    let mut conversation = Conversation::default();
    assert!(conversation.begin(" ", context(&session)).is_err());
    assert!(
        conversation
            .begin(&"x".repeat(MAX_PROMPT + 1), context(&session))
            .is_err()
    );
    let first = conversation.begin("Explain this sheet", context(&session))?;
    assert!(conversation.begin("second", context(&session)).is_err());
    assert!(conversation.receive(first.turn, Event::Text("First Δ".into())));
    conversation.stop();
    assert_eq!(
        conversation.messages().last().map(|m| &m.status),
        Some(&Status::Stopped)
    );
    let second = conversation.begin("Continue", context(&session))?;
    assert!(
        second
            .messages
            .iter()
            .any(|m| m.text == "First Δ" && m.status == Status::Stopped)
    );
    assert!(!conversation.receive(first.turn, Event::Text("must not appear".into())));
    conversation.receive(second.turn, Event::Text("Second answer".into()));
    conversation.receive(second.turn, Event::Finished);
    assert_eq!(
        conversation.messages().last().map(|m| m.text.as_str()),
        Some("Second answer")
    );
    conversation.clear();
    let third = conversation.begin("New conversation", context(&session))?;
    assert!(third.turn > second.turn);
    assert!(!conversation.receive(second.turn, Event::Finished));
    assert_eq!(third.messages.len(), 1);
    Ok(())
}

#[test]
fn conversation_bounds_history_and_reports_output_limits_and_revision_changes() -> Result<()> {
    let mut session = session()?;
    let mut conversation = Conversation::default();
    for _ in 0..40 {
        let request = conversation.begin("Keep this circuit", context(&session))?;
        conversation.receive(request.turn, Event::Text("α".repeat(MAX_REPLY / 2)));
        conversation.receive(request.turn, Event::Finished);
        assert!(conversation.messages().len() <= MAX_MESSAGES);
        assert!(
            conversation
                .messages()
                .iter()
                .map(|m| m.text.len())
                .sum::<usize>()
                <= MAX_HISTORY
        );
    }
    let request = conversation.begin("Too much output", context(&session))?;
    conversation.receive(request.turn, Event::Text("x".repeat(MAX_REPLY + 1)));
    assert!(matches!(
        conversation.messages().last().map(|m| &m.status),
        Some(Status::Failed(_))
    ));
    let request = conversation.begin("Explain", context(&session))?;
    session.apply(
        session.revision(),
        Command::Place {
            kind: PartKind::Resistor,
            at: Point { x: 0, y: 0 },
        },
    )?;
    assert!(conversation.invalidate(&session.snapshot()));
    assert!(!conversation.receive(request.turn, Event::Text("outdated".into())));
    // Undo restores geometry, but advances the revision and still invalidates old work.
    session.undo()?;
    assert_ne!(request.context.snapshot, session.snapshot());
    let reopened = Session::from_project(persistence::decode_project(
        &persistence::encode_project(session.project())?,
    )?)?;
    assert_ne!(request.context.snapshot, reopened.snapshot());
    Ok(())
}

#[test]
fn conversation_tools_share_proposals_review_history_and_reopen() -> Result<()> {
    let mut session = session()?;
    let before = session.project().clone();
    let context = context(&session);
    let reply=dispatch(&mut session,&context,mcp::Call {name:"prepare_edit".into(),arguments:json!({
        "expected":context.snapshot,"sheet":context.sheet,"summary":"Add a generic resistor",
        "operations":[{"op":"place","key":"r","kind":"resistor","at":{"x":0,"y":0}}]
    })}).map_err(|e|Error::Invalid(e.message))?;
    let handle: ProjectId = serde_json::from_value(reply["handle"].clone())
        .map_err(|e| Error::Invalid(e.to_string()))?;
    assert_eq!(session.project(), &before);
    assert!(
        dispatch(
            &mut session,
            &context,
            mcp::Call {
                name: "apply_edit".into(),
                arguments: json!({})
            }
        )
        .is_err()
    );
    assert!(
        dispatch(
            &mut session,
            &context,
            mcp::Call {
                name: "discard_edit".into(),
                arguments: json!({"handle":handle})
            }
        )
        .is_err()
    );
    session.apply_proposal(Apply {
        expected: context.snapshot.clone(),
        handle,
        request_id: "gui-review".into(),
    })?;
    assert_eq!(session.document().symbols.len(), 1);
    let reopened = Session::from_project(persistence::decode_project(
        &persistence::encode_project(session.project())?,
    )?)?;
    assert_eq!(reopened.project(), session.project());
    assert_eq!(
        dispatch(
            &mut session,
            &context,
            mcp::Call {
                name: "inspect_project".into(),
                arguments: json!({})
            }
        )
        .err()
        .map(|e| e.code),
        Some("conflict".into())
    );
    session.undo()?;
    assert_eq!(session.project(), &before);
    Ok(())
}

struct Scripted;
impl Backend for Scripted {
    fn run(self: Box<Self>, _request: Request, stream: &Stream) -> Result<()> {
        stream.text("A streamed ".into())?;
        stream.text("reply.".into())
    }
}
#[test]
fn backend_worker_delivers_text_and_one_terminal_result() -> Result<()> {
    let mut conversation = Conversation::default();
    let request = conversation.begin("Explain", context(&session()?))?;
    let worker = Worker::start(Box::new(Scripted), request, || {})?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while conversation.active().is_some() && Instant::now() < deadline {
        if let Some(event) = worker.next() {
            conversation.receive(worker.turn, event);
        } else {
            std::thread::yield_now();
        }
    }
    assert!(conversation.active().is_none());
    assert_eq!(
        conversation.messages().last().map(|m| m.text.as_str()),
        Some("A streamed reply.")
    );
    assert_eq!(
        conversation.messages().last().map(|m| &m.status),
        Some(&Status::Complete)
    );
    Ok(())
}
