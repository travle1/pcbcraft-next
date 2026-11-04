use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn id() -> Value {
    json!({"type":"array","items":{"type":"integer","minimum":0,"maximum":255},"minItems":16,"maxItems":16,"description":"Opaque ID. Copy the 16-byte array from inspection; never infer it from a reference label."})
}
fn point() -> Value {
    object(
        json!({"x":{"type":"integer","minimum":-1000000000,"maximum":1000000000},"y":{"type":"integer","minimum":-1000000000,"maximum":1000000000}}),
        &["x", "y"],
    )
}
fn symbol() -> Value {
    json!({"oneOf":[object(json!({"id":id()}),&["id"]),object(json!({"local":{"type":"string","minLength":1,"maxLength":64}}),&["local"])]})
}
fn terminal() -> Value {
    object(
        json!({"symbol":symbol(),"terminal":{"type":"string","description":"Exact string pin ID from list_parts, not the pin's name."}}),
        &["symbol", "terminal"],
    )
}
fn snapshot() -> Value {
    object(
        json!({"project":id(),"incarnation":id(),"revision":{"type":"integer","minimum":0}}),
        &["project", "incarnation", "revision"],
    )
}
fn operation(name: &str, mut properties: Value, required: &[&str]) -> Value {
    properties["op"] = json!({"const":name,"type":"string"});
    let mut required = required.to_vec();
    required.push("op");
    object(properties, &required)
}
fn operations() -> Value {
    let text = json!({"type":"string"});
    json!({"type":"array","minItems":1,"maxItems":64,"items":{"oneOf":[
        operation("place",json!({"key":{"type":"string","minLength":1,"maxLength":64},"kind":{"type":"string","enum":crate::document::PartKind::ALL},"at":point(),"reference":text,"value":text,"quarter_turns":{"type":"integer","minimum":0,"maximum":3}}),&["key","kind","at"]),
        operation("move",json!({"symbol":symbol(),"at":point()}),&["symbol","at"]),
        operation("edit",json!({"symbol":symbol(),"reference":text,"value":text,"quarter_turns":{"type":"integer","minimum":0,"maximum":3}}),&["symbol","reference","value","quarter_turns"]),
        operation("delete",json!({"symbol":symbol()}),&["symbol"]),
        operation("connect",json!({"from":terminal(),"to":terminal(),"bends":{"type":"array","maxItems":256,"items":point(),"description":"Optional explicit intermediate orthogonal points. Omit to use bounded body/pin-aware routing."}}),&["from","to"]),
        operation("disconnect",json!({"wire":id()}),&["wire"]),
        operation("rename_net",json!({"anchor":terminal(),"name":text}),&["anchor","name"]),
        operation("note",json!({"at":point(),"text":text}),&["at","text"]),
        operation("arrange",json!({"symbols":{"type":"array","minItems":1,"maxItems":64,"items":symbol()},"origin":point(),"columns":{"type":"integer","minimum":1,"maximum":8},"spacing":point()}),&["symbols","origin","columns","spacing"])
    ]}})
}
pub fn tools() -> Vec<Value> {
    let page = json!({"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":200,"default":100}});
    let mut sheet = page.clone();
    sheet["sheet"] = id();
    let specs = vec![
        (
            "pcb_reference",
            "Read release-specific capture guidance and honest capability limits",
            object(json!({}), &[]),
            true,
        ),
        (
            "inspect_project",
            "Inspect the live project identity, revision, sheet occurrences, GUI selection and capabilities",
            object(json!({}), &[]),
            true,
        ),
        (
            "list_parts",
            "List original built-in symbol definitions and exact terminal IDs; these are generic modules, not real manufacturer pinouts",
            object(json!({}), &[]),
            true,
        ),
        (
            "inspect_sheet",
            "Read a paginated schematic occurrence with placements, components, pins, wires, notes and sheet interfaces",
            object(sheet, &["sheet"]),
            true,
        ),
        (
            "inspect_nets",
            "Inspect occurrence-qualified logical connectivity, including singleton/unconnected terminals. This is not ERC",
            object(page, &[]),
            true,
        ),
        (
            "prepare_edit",
            "Prepare one atomic candidate without editing the live document. Local keys reference earlier placements. Changes to a shared definition affect all its occurrences. Arrange before connecting; visually review the candidate",
            object(
                json!({"expected":snapshot(),"sheet":id(),"summary":{"type":"string","minLength":1,"maxLength":240},"operations":operations()}),
                &["expected", "sheet", "summary", "operations"],
            ),
            false,
        ),
        (
            "inspect_proposal",
            "Read a server-owned proposal, its stale state, affected occurrences and candidate sheet (first 200 items, explicit truncation)",
            object(json!({"handle":id()}), &["handle"]),
            true,
        ),
        (
            "apply_edit",
            "Commit a prepared proposal once through GUI history. Requires external apply enabled by the user in AI / MCP; otherwise the user applies in the panel. Reuse exactly the same request_id and arguments after uncertain completion",
            object(
                json!({"request_id":{"type":"string","minLength":1,"maxLength":128},"expected":snapshot(),"handle":id()}),
                &["request_id", "expected", "handle"],
            ),
            false,
        ),
        (
            "discard_edit",
            "Discard a pending proposal without touching the document",
            object(json!({"handle":id()}), &["handle"]),
            false,
        ),
        (
            "edit_status",
            "Look up a commit receipt and pending proposals. Receipts survive undo and are retained for this runtime session only",
            object(
                json!({"request_id":{"type":"string","maxLength":128}}),
                &["request_id"],
            ),
            true,
        ),
    ];
    specs.into_iter().map(|(name,description,schema,read_only)|json!({"name":name,"description":description,"inputSchema":schema,"annotations":{"readOnlyHint":read_only,"destructiveHint":name=="apply_edit","idempotentHint":name!="prepare_edit","openWorldHint":false}})).collect()
}
