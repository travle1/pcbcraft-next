use super::*;
#[test]
fn lifecycle_notifications_errors_and_capabilities() {
    let mut p = Protocol::default();
    let mut calls = 0;
    let mut tool = |_: Call| {
        calls += 1;
        Ok(json!({"ok":true}))
    };
    let response = p.receive(
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        &mut tool,
    );
    assert_eq!(
        response.and_then(|v| v["error"]["code"].as_i64()),
        Some(-32002)
    );
    let init=p.receive(json!({"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"future-version","capabilities":{},"clientInfo":{"name":"original-test","version":"1"}}}),&mut tool);
    assert_eq!(
        init.map(|v| v["result"]["protocolVersion"].clone()),
        Some(json!(PROTOCOL))
    );
    assert!(
        p.receive(
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            &mut tool
        )
        .is_none()
    );
    let list = p.receive(
        json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
        &mut tool,
    );
    assert_eq!(
        list.map(|v| v["result"]["tools"].as_array().map(Vec::len)),
        Some(Some(10))
    );
    assert!(p.receive(json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"apply_edit","arguments":{}}}),&mut tool).is_none());
    assert_eq!(calls, 0);
}
#[test]
fn stdio_is_only_json_and_bounded() -> std::io::Result<()> {
    let input = b"not json\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
    let mut output = vec![];
    serve(&mut &input[..], &mut output, |_| Ok(json!({})))?;
    let replies = output
        .split(|b| *b == b'\n')
        .filter(|b| !b.is_empty())
        .map(serde_json::from_slice::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["error"]["code"], -32700);
    assert_eq!(replies[1]["result"], json!({}));
    assert!(read_frame(&mut &b"abcdefgh\n"[..], 4).is_err());
    assert!(read_frame(&mut &b"unterminated"[..], 64).is_err());
    Ok(())
}
#[test]
fn live_dispatch_catalog_paging_and_review_gate() -> Result<()> {
    let mut s = Session::demo()?;
    let ctx = Context {
        active_sheet: s.project().root,
        selection: json!({}),
        allow_apply: false,
    };
    let invoke = |s: &mut Session, name: &str, args: Value| {
        dispatch(
            s,
            &ctx,
            Call {
                name: name.into(),
                arguments: args,
            },
        )
        .map_err(|e| Error::Invalid(e.message))
    };
    let project = invoke(&mut s, "inspect_project", json!({}))?;
    assert_eq!(
        project["snapshot"],
        serde_json::to_value(s.snapshot()).map_err(|e| e.to_string())?
    );
    let sheet = s.project().root;
    let page = invoke(&mut s, "inspect_sheet", json!({"sheet":sheet,"limit":2}))?;
    assert_eq!(page["items"].as_array().map(Vec::len), Some(2));
    assert_eq!(page["next_offset"], 2);
    assert!(invoke(&mut s, "inspect_sheet", json!({"sheet":sheet,"limit":201})).is_err());
    let catalog = invoke(&mut s, "list_parts", json!({}))?;
    assert_eq!(catalog["parts"].as_array().map(Vec::len), Some(12));
    let snapshot = s.snapshot();
    let plan = invoke(
        &mut s,
        "prepare_edit",
        json!({"expected":snapshot,"sheet":sheet,"summary":"Add a note","operations":[{"op":"note","at":{"x":0,"y":0},"text":"Review me"}]}),
    )?;
    assert_eq!(s.revision(), 0);
    let apply = json!({"request_id":"test","expected":snapshot,"handle":plan["handle"]});
    assert!(invoke(&mut s, "apply_edit", apply.clone()).is_err());
    assert_eq!(s.revision(), 0);
    let allowed = Context {
        allow_apply: true,
        ..ctx
    };
    let applied = dispatch(
        &mut s,
        &allowed,
        Call {
            name: "apply_edit".into(),
            arguments: apply.clone(),
        },
    );
    assert!(applied.is_ok());
    assert_eq!(s.revision(), 1);
    assert!(
        dispatch(
            &mut s,
            &allowed,
            Call {
                name: "apply_edit".into(),
                arguments: apply
            }
        )
        .is_ok()
    );
    assert_eq!(s.revision(), 1);
    Ok(())
}
