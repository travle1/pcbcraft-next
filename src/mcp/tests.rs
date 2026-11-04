use super::*;
#[test]
fn connected_arrangement_is_discoverable_and_uses_normal_proposal_history() -> Result<()> {
    let mut s = Session::new(Document::empty("Original MCP arrangement")?)?;
    let context = Context {
        active_sheet: s.project().root,
        selection: json!({}),
        allow_apply: true,
    };
    let invoke = |s: &mut Session, name: &str, args: Value| {
        dispatch(
            s,
            &context,
            Call {
                name: name.into(),
                arguments: args,
            },
        )
        .map_err(|e| Error::Invalid(e.message))
    };
    let capabilities = invoke(&mut s, "inspect_project", json!({}))?;
    assert_eq!(
        capabilities["capabilities"]["connected_grid_arrangement"],
        true
    );
    let tools = schema::tools();
    let tool = tools
        .iter()
        .find(|t| t["name"] == "prepare_edit")
        .ok_or("Missing tool")?;
    let variants = tool["inputSchema"]["properties"]["operations"]["items"]["oneOf"]
        .as_array()
        .ok_or("Missing operations")?;
    assert!(
        variants
            .iter()
            .any(|op| op["properties"]["op"]["const"] == "arrange_connected")
    );
    let point = |x: i64, y: i64| json!({"x":x*GRID_NM,"y":y*GRID_NM});
    let args = json!({"expected":s.snapshot(),"sheet":context.active_sheet,"summary":"Place and arrange a connected pair","operations":[
        {"op":"place","key":"a","kind":"resistor","at":point(0,-40)},
        {"op":"place","key":"b","kind":"led","at":point(80,-40)},
        {"op":"connect","from":{"symbol":{"local":"a"},"terminal":"2"},"to":{"symbol":{"local":"b"},"terminal":"1"}},
        {"op":"arrange_connected","symbols":[{"local":"a"},{"local":"b"}],"locked":[],"region":{"min":point(-20,-20),"max":point(80,20)},"origin":point(0,0),"columns":2,"spacing":point(40,40)}
    ]});
    let mut bad = args.clone();
    bad["operations"][3]["region"]["unexpected"] = json!(true);
    assert!(invoke(&mut s, "prepare_edit", bad).is_err());
    assert!(s.proposals().is_empty());
    let proposal = invoke(&mut s, "prepare_edit", args)?;
    assert_eq!(s.revision(), 0);
    let handle = serde_json::from_value(proposal["handle"].clone()).map_err(|e| e.to_string())?;
    let candidate = s.proposal_project(handle)?.clone();
    let apply = json!({"request_id":"connected-grid","expected":s.snapshot(),"handle":handle});
    invoke(&mut s, "apply_edit", apply.clone())?;
    assert_eq!(s.project(), &candidate);
    invoke(&mut s, "apply_edit", apply)?;
    assert_eq!(s.revision(), 1);
    s.undo()?;
    assert!(s.document().symbols.is_empty());
    assert!(s.document().wires.is_empty());
    Ok(())
}

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
