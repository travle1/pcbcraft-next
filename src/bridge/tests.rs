use super::*;
#[test]
fn bridge_reaches_owner_and_cleans_up() -> std::io::Result<()> {
    let (wake_tx, wake_rx) = mpsc::channel();
    let bridge = Bridge::start(move || {
        let _ = wake_tx.send(());
    })?;
    let path = bridge.path();
    let connection = connect(&path)?;
    let worker = std::thread::spawn(move || {
        connection.call(Call {
            name: "inspect_project".into(),
            arguments: serde_json::json!({}),
        })
    });
    wake_rx
        .recv_timeout(Duration::from_secs(3))
        .map_err(std::io::Error::other)?;
    let request = bridge
        .next()
        .ok_or_else(|| std::io::Error::other("Missing request"))?;
    assert_eq!(request.call.name, "inspect_project");
    request.reply(Ok(serde_json::json!({"same_session":true})));
    let reply = worker
        .join()
        .map_err(|_| std::io::Error::other("Worker failed"))?
        .map_err(|e| std::io::Error::other(e.message))?;
    assert_eq!(reply["same_session"], true);
    drop(bridge);
    assert!(!path.exists());
    Ok(())
}
#[test]
fn bad_token_never_reaches_session() -> std::io::Result<()> {
    let bridge = Bridge::start(|| {})?;
    let mut connection = connect(&bridge.path())?;
    connection.token = "0".repeat(64);
    let result = connection.call(Call {
        name: "inspect_project".into(),
        arguments: serde_json::json!({}),
    });
    assert!(matches!(result,Err(e) if e.code=="unauthorized"));
    assert!(bridge.next().is_none());
    Ok(())
}
