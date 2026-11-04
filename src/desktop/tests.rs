use super::*;

fn measured_area(ctx: &egui::Context, name: &'static str) -> Result<egui::Rect> {
    egui::containers::AreaState::load(ctx, name.into())
        .map(|s| s.rect())
        .ok_or_else(|| format!("Missing area {name}").into())
}

#[test]
fn workspace_panels_respect_real_bounds_and_drawer_does_not_move_camera() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    for size in [egui::vec2(1600.0, 1000.0), egui::vec2(1000.0, 700.0)] {
        for dock in [
            library::Dock::Left,
            library::Dock::Right,
            library::Dock::Floating,
        ] {
            app.library_ui.dock = dock;
            app.library = true;
            app.drawer = None;
            let run = |app: &mut App| {
                ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| app.show(ui),
                )
            };
            for _ in 0..3 {
                run(&mut app).drop_without_applying_deltas();
            }
            let offset = app.canvas.offset;
            let zoom = app.canvas.zoom;
            app.drawer = Some("Messages");
            app.tool = Tool::Wire;
            for _ in 0..3 {
                run(&mut app).drop_without_applying_deltas();
            }
            assert_eq!(
                offset, app.canvas.offset,
                "Opening a drawer must not move the camera"
            );
            assert_eq!(zoom, app.canvas.zoom);
            let drawer = measured_area(&ctx, "bottom_drawer")?;
            let nav = measured_area(&ctx, "navigation")?;
            let hint = measured_area(&ctx, "tool_hint")?;
            let library = measured_area(&ctx, "parts_library")?;
            let browser = measured_area(&ctx, "browser")?;
            let properties = measured_area(&ctx, "properties")?;
            let filters = measured_area(&ctx, "display_controls")?;
            for (name, rect) in [
                ("drawer", drawer),
                ("navigation", nav),
                ("hint", hint),
                ("library", library),
                ("browser", browser),
                ("properties", properties),
                ("filters", filters),
            ] {
                assert!(
                    app.layout.bounds.expand(1.0).contains_rect(rect),
                    "{name}: {rect:?} outside {:?}",
                    app.layout.bounds
                );
            }
            assert!(
                !drawer.intersects(nav) && !drawer.intersects(hint) && !drawer.intersects(filters)
            );
            assert!(
                !drawer.intersects(library),
                "Library must not overflow into drawer: {library:?}, {drawer:?}"
            );
            assert!(!properties.intersects(filters));
            if dock == library::Dock::Left {
                assert!(!library.intersects(browser));
            }
            if dock == library::Dock::Right {
                assert!(!library.intersects(properties) && !library.intersects(filters));
            }
        }
    }
    Ok(())
}

#[test]
fn library_header_floats_snaps_and_cancels_without_document_changes() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    let before = app.document().clone();
    let revision = app.session.revision();
    let move_header = |app: &mut App, target: egui::Pos2, cancel: bool| -> Result<()> {
        let origin = measured_area(&ctx, "parts_library")?.min + egui::vec2(90.0, 24.0);
        workspace_frame(
            app,
            &ctx,
            vec![
                egui::Event::PointerMoved(origin),
                egui::Event::PointerButton {
                    pos: origin,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
        workspace_frame(app, &ctx, vec![egui::Event::PointerMoved(target)])
            .drop_without_applying_deltas();
        assert!(app.library_ui.moving());
        if cancel {
            workspace_key(app, &ctx, egui::Key::Escape);
        }
        release_library_drag(app, &ctx, target);
        Ok(())
    };
    move_header(&mut app, egui::pos2(20.0, 280.0), false)?;
    assert_eq!(app.library_ui.dock, library::Dock::Left);
    move_header(&mut app, egui::pos2(980.0, 280.0), false)?;
    assert_eq!(app.library_ui.dock, library::Dock::Right);
    move_header(&mut app, egui::pos2(480.0, 280.0), true)?;
    assert_eq!(app.library_ui.dock, library::Dock::Right);
    assert!(
        app.library,
        "Escape cancels a panel drag without closing the library"
    );
    move_header(&mut app, egui::pos2(480.0, 280.0), false)?;
    assert_eq!(app.library_ui.dock, library::Dock::Floating);
    assert_eq!(app.document(), &before);
    assert_eq!(app.session.revision(), revision);
    Ok(())
}

#[test]
fn command_search_keyboard_dispatch_and_dismissal_preserve_canvas() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.tool = Tool::Place(PartKind::Led);
    let before = app.document().clone();
    app.palette = true;
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    workspace_key(&mut app, &ctx, egui::Key::Escape);
    assert!(!app.palette);
    assert!(app.tool == Tool::Place(PartKind::Led));
    app.palette = true;
    app.query = "wire".into();
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    workspace_key(&mut app, &ctx, egui::Key::Enter);
    assert!(
        !app.palette && app.tool == Tool::Wire,
        "Enter should run the selected command"
    );
    app.palette = true;
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    workspace_key(&mut app, &ctx, egui::Key::ArrowDown);
    assert_eq!(app.palette_ui.selected, 1);
    workspace_key(&mut app, &ctx, egui::Key::Tab);
    assert_eq!(app.query, "Draw wire");
    workspace_key(&mut app, &ctx, egui::Key::Enter);
    assert!(!app.palette && app.tool == Tool::Wire);
    app.palette = true;
    app.tool = Tool::Place(PartKind::Led);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    workspace_click(&mut app, &ctx, egui::pos2(600.0, 600.0));
    assert!(!app.palette);
    assert_eq!(
        app.document(),
        &before,
        "Dismissal must not place a component"
    );
    Ok(())
}

pub(super) fn library_fixture() -> Result<(App, egui::Context)> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: true,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    for _ in 0..4 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    Ok((app, ctx))
}

fn library_text(output: &egui::FullOutput, label: &str) -> Result<egui::Rect> {
    let heading = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Parts library" => Some(text.pos),
            _ => None,
        })
        .ok_or("Missing library heading")?;
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text)
                if text.galley.job.text == label
                    && text.pos.x > heading.x - 24.0
                    && text.pos.x < heading.x + 250.0
                    && text.pos.y > heading.y =>
            {
                let rect = text.galley.rect.translate(text.pos.to_vec2());
                shape.clip_rect.contains_rect(rect).then_some(rect)
            }
            _ => None,
        })
        .ok_or_else(|| format!("Missing/unreadable library text: {label}").into())
}

#[test]
fn parts_library_modes_search_and_click_placement() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    let before = app.document().clone();
    let revision = app.session.revision();
    for (name, view) in [
        ("Small", library::View::Small),
        ("Medium", library::View::Medium),
        ("Large", library::View::Large),
        ("List", library::View::List),
    ] {
        let output = workspace_frame(&mut app, &ctx, vec![]);
        let mode = library_text(&output, name)?.center();
        output.drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, mode);
        assert!(app.library_ui.view == view);
        let output = workspace_frame(&mut app, &ctx, vec![]);
        let first = library_text(&output, "Resistor")?;
        if view != library::View::Large {
            let second = library_text(&output, "Capacitor")?;
            if view == library::View::List {
                assert!(second.top() > first.bottom());
            } else {
                assert!(second.left() > first.right());
            }
        }
        output.drop_without_applying_deltas();
        assert_eq!(app.document(), &before);
        assert_eq!(app.session.revision(), revision);
    }
    // Filtering is case insensitive and searches both part name and value.
    app.library_ui.search = "gReEn".into();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let led = library_text(&output, "LED")?.center();
    assert!(library_text(&output, "1 part").is_ok());
    assert!(library_text(&output, "Resistor").is_err());
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, led);
    assert!(app.library && app.tool == Tool::Place(PartKind::Led));
    assert_eq!(
        app.document(),
        &before,
        "Choosing a thumbnail only starts a preview"
    );
    workspace_click(&mut app, &ctx, egui::pos2(600.0, 500.0));
    assert_eq!(app.document().components.len(), before.components.len() + 1);
    assert_eq!(app.session.revision(), revision + 1);
    assert!(app.library, "The parts browser stays open after placement");
    app.history(false);
    assert_eq!(app.document(), &before);
    app.library_ui.search = "no matching original part".into();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    assert!(library_text(&output, "No matching parts").is_ok());
    let clear = library_text(&output, "Clear search")?.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, clear);
    assert!(app.library_ui.search.is_empty());
    Ok(())
}

fn begin_library_drag(app: &mut App, ctx: &egui::Context) -> Result<()> {
    let output = workspace_frame(app, ctx, vec![]);
    let origin = library_text(&output, "Resistor")?.center();
    output.drop_without_applying_deltas();
    workspace_frame(
        app,
        ctx,
        vec![
            egui::Event::PointerMoved(origin),
            egui::Event::PointerButton {
                pos: origin,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    )
    .drop_without_applying_deltas();
    workspace_frame(
        app,
        ctx,
        vec![egui::Event::PointerMoved(origin + egui::vec2(20.0, 0.0))],
    )
    .drop_without_applying_deltas();
    assert!(egui::DragAndDrop::has_payload_of_type::<library::PartDrag>(
        ctx
    ));
    Ok(())
}

fn release_library_drag(app: &mut App, ctx: &egui::Context, target: egui::Pos2) {
    workspace_frame(app, ctx, vec![egui::Event::PointerMoved(target)])
        .drop_without_applying_deltas();
    workspace_frame(
        app,
        ctx,
        vec![egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    )
    .drop_without_applying_deltas();
    workspace_frame(app, ctx, vec![]).drop_without_applying_deltas();
}

#[test]
fn library_drag_placement_is_atomic_cancellable_and_revision_bound() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    let before = app.document().clone();
    let revision = app.session.revision();
    begin_library_drag(&mut app, &ctx)?;
    assert_eq!(app.document(), &before);
    release_library_drag(&mut app, &ctx, egui::pos2(600.0, 500.0));
    assert_eq!(app.session.revision(), revision + 1);
    assert_eq!(app.document().components.len(), before.components.len() + 1);
    app.history(false);
    assert_eq!(app.document(), &before);
    let revision = app.session.revision();
    begin_library_drag(&mut app, &ctx)?;
    release_library_drag(&mut app, &ctx, egui::pos2(80.0, 240.0));
    assert_eq!(
        app.session.revision(),
        revision,
        "Dropping over the library is not placement"
    );
    begin_library_drag(&mut app, &ctx)?;
    workspace_key(&mut app, &ctx, egui::Key::Escape);
    release_library_drag(&mut app, &ctx, egui::pos2(600.0, 500.0));
    assert_eq!(app.document(), &before);
    begin_library_drag(&mut app, &ctx)?;
    let selected = app.document().symbols.first().ok_or("Missing symbol")?.id;
    app.command(Command::Rotate { symbol: selected });
    let changed = app.document().clone();
    let revision = app.session.revision();
    release_library_drag(&mut app, &ctx, egui::pos2(600.0, 500.0));
    assert_eq!(
        app.document(),
        &changed,
        "Stale drag cannot modify the new revision"
    );
    assert_eq!(app.session.revision(), revision);
    Ok(())
}

fn ribbon_frame(
    app: &mut App,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 700.0),
            )),
            events,
            ..Default::default()
        },
        |ui| app.ribbon(ui),
    )
}

pub(super) fn ribbon_text(
    output: &egui::FullOutput,
    label: &str,
) -> Result<(egui::Rect, egui::Rect)> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => Some((
                text.galley.rect.translate(text.pos.to_vec2()),
                shape.clip_rect,
            )),
            _ => None,
        })
        .ok_or_else(|| format!("Missing ribbon text: {label}").into())
}

#[test]
fn ribbon_rows_remain_aligned_and_unclipped_at_supported_sizes() -> Result<()> {
    for (width, text_scale, pixels_per_point) in [
        (1000.0, 1.0, 1.0),
        (1600.0, 1.0, 1.0),
        (1600.0, 1.25, 1.0),
        (1000.0, 1.0, 1.5),
    ] {
        let mut app = App::create(Options {
            demo: true,
            capture: None,
            exit_after_capture: false,
            capture_workspace: false,
            capture_library: false,
            capture_panels: false,
            capture_search: false,
            capture_properties: false,
            capture_nets: false,
            capture_wire_menu: false,
            capture_text: false,
            capture_selection: false,
            open: None,
        })?;
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        ctx.set_pixels_per_point(pixels_per_point);
        let mut t = theme::Theme::default();
        t.metrics.body *= text_scale;
        t.metrics.small *= text_scale;
        t.metrics.control_height *= text_scale;
        theme::set(&ctx, t);
        for _ in 0..3 {
            ribbon_frame(&mut app, &ctx, width, vec![]).drop_without_applying_deltas();
        }
        let output = ribbon_frame(&mut app, &ctx, width, vec![]);
        let cards: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == t.ui.surface && rect.rect.height() > 75.0 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            cards.len(),
            2,
            "Workspace and commands must have distinct cards"
        );
        assert_eq!(cards[0].top(), cards[1].top());
        assert_eq!(cards[0].bottom(), cards[1].bottom());
        let caption_y = ribbon_text(&output, "Place")?.0.center().y;
        for (caption, tool) in [
            ("Place", "Symbol"),
            ("Sheets", "Subsheet"),
            ("Connect", "Wire"),
            ("Modify", "Move"),
            ("Inspect", "ERC"),
            ("Design", "Sync"),
        ] {
            let (caption, clip) = ribbon_text(&output, caption)?;
            let (tool, tool_clip) = ribbon_text(&output, tool)?;
            assert!(
                clip.contains_rect(caption),
                "Caption clipped at width {width}, scale {text_scale}"
            );
            assert!(tool_clip.contains_rect(tool), "Command label clipped");
            assert!((caption.center().y - caption_y).abs() < 0.5);
            assert!(
                caption.top() - tool.bottom() >= 8.0,
                "Captions need their own row"
            );
            assert!(
                cards[1].bottom() - caption.bottom() >= 6.0,
                "Caption crowds card edge"
            );
        }
        for label in [
            "Capture",
            "Analyze",
            "Libraries",
            "Manage",
            "View",
            "Search commands",
            "Workspace",
            "Schematic",
        ] {
            let (text, clip) = ribbon_text(&output, label)?;
            assert!(clip.contains_rect(text), "Clipped {label}");
        }
        let mut previous_x = 0.0;
        for label in [
            "Symbol",
            "Power",
            "Port",
            "New sheet",
            "Subsheet",
            "Wire",
            "Bus",
            "Label",
            "Move",
            "Rotate",
            "Mirror",
            "Rules",
            "ERC",
            "Annotate",
            "BOM",
            "Sync",
        ] {
            if width == 1000.0 && ["Power", "Bus", "Rules", "BOM"].contains(&label) {
                assert!(ribbon_text(&output, label).is_err());
                continue;
            }
            let (text, clip) = ribbon_text(&output, label)?;
            assert!(clip.contains_rect(text), "Clipped command: {label}");
            assert!(
                text.center().x > previous_x,
                "Capture commands must retain the reference order"
            );
            previous_x = text.center().x;
        }
        output.drop_without_applying_deltas();
    }
    Ok(())
}

#[test]
fn capture_group_menu_shares_commands_and_disables_planned_edits() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    let click = |app: &mut App, position| {
        for pressed in [true, false] {
            ribbon_frame(
                app,
                &ctx,
                1000.0,
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .drop_without_applying_deltas();
        }
        ribbon_frame(app, &ctx, 1000.0, vec![]).drop_without_applying_deltas();
    };
    for _ in 0..4 {
        ribbon_frame(&mut app, &ctx, 1000.0, vec![]).drop_without_applying_deltas();
    }
    let output = ribbon_frame(&mut app, &ctx, 1000.0, vec![]);
    let modify = ribbon_text(&output, "Modify")?.0.center();
    output.drop_without_applying_deltas();
    click(&mut app, modify);
    let menu_item = |output: &egui::FullOutput, label: &str| -> Result<egui::Pos2> {
        output
            .shapes
            .iter()
            .rev()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == label => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()).center())
                }
                _ => None,
            })
            .ok_or_else(|| format!("Missing menu item: {label}").into())
    };
    let output = ribbon_frame(&mut app, &ctx, 1000.0, vec![]);
    let mirror = menu_item(&output, "Mirror")?;
    assert!(
        menu_item(&output, "Delete").is_ok(),
        "Delete remains in Modify's menu"
    );
    output.drop_without_applying_deltas();
    let before = app.document().clone();
    let revision = app.session.revision();
    click(&mut app, mirror);
    assert_eq!(app.document(), &before);
    assert_eq!(app.session.revision(), revision);
    assert!(
        egui::Popup::is_any_open(&ctx),
        "Disabled row leaves its menu open"
    );
    let output = ribbon_frame(&mut app, &ctx, 1000.0, vec![]);
    let rotate = menu_item(&output, "Rotate")?;
    output.drop_without_applying_deltas();
    click(&mut app, rotate);
    assert_eq!(app.session.revision(), revision + 1);
    assert!(!egui::Popup::is_any_open(&ctx));
    app.history(false);
    assert_eq!(
        app.document(),
        &before,
        "Dropdown edit uses the existing session undo"
    );
    Ok(())
}

#[test]
fn schematic_grid_and_fit_are_presentation_only() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    let t = theme::current(&ctx);
    let before = app.document().clone();
    let revision = app.session.revision();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let grid_count = |output: &egui::FullOutput| {
        output.shapes.iter().filter(|shape| matches!(shape.shape,
        egui::Shape::LineSegment { stroke, .. } if stroke.color == t.canvas.grid || stroke.color == t.canvas.grid_major
    )).count()
    };
    assert!(
        (10..500).contains(&grid_count(&output)),
        "Adaptive grid draws bounded line geometry"
    );
    assert!(
        app.canvas.zoom <= 0.75,
        "Fit must not magnify a small draft to fill the screen"
    );
    output.drop_without_applying_deltas();
    app.canvas.grid = false;
    let output = workspace_frame(&mut app, &ctx, vec![]);
    assert_eq!(grid_count(&output), 0);
    assert_eq!(app.document(), &before);
    assert_eq!(app.session.revision(), revision);
    output.drop_without_applying_deltas();
    Ok(())
}

#[test]
fn ribbon_pointer_targets_dispatch_the_existing_actions() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    for _ in 0..3 {
        ribbon_frame(&mut app, &ctx, 1000.0, vec![]).drop_without_applying_deltas();
    }
    for label in [
        "Analyze",
        "ERC",
        "Capture",
        "Symbol",
        "Wire",
        "Search commands",
        "Workspace",
    ] {
        let output = ribbon_frame(&mut app, &ctx, 1000.0, vec![]);
        let position = ribbon_text(&output, label)?.0.center();
        output.drop_without_applying_deltas();
        for pressed in [true, false] {
            ribbon_frame(
                &mut app,
                &ctx,
                1000.0,
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .drop_without_applying_deltas();
        }
    }
    assert_eq!(app.drawer, Some("ERC"));
    assert!(app.library);
    assert!(app.tool == Tool::Wire);
    assert!(app.palette);
    let output = ribbon_frame(&mut app, &ctx, 1000.0, vec![]);
    assert!(
        ribbon_text(&output, "PCB layout").is_ok(),
        "Workspace menu must open"
    );
    output.drop_without_applying_deltas();
    Ok(())
}

pub(super) fn workspace_frame(
    app: &mut App,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 700.0),
            )),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    )
}

pub(super) fn workspace_click(app: &mut App, ctx: &egui::Context, pos: egui::Pos2) {
    for pressed in [true, false] {
        workspace_frame(
            app,
            ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
    }
    workspace_frame(app, ctx, vec![]).drop_without_applying_deltas();
}

fn workspace_key(app: &mut App, ctx: &egui::Context, key: egui::Key) {
    for pressed in [true, false] {
        workspace_frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        )
        .drop_without_applying_deltas();
    }
}

#[test]
fn workspace_picker_dismissal_and_planned_rows_preserve_editor_state() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    for _ in 0..4 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    app.tool = Tool::Place(PartKind::Resistor);
    let document = app.document().clone();
    let revision = app.session.revision();
    let selected = app.selected_symbol();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    // Open through the icon area, not only the tiny Workspace caption.
    let opener = ribbon_text(&output, "Schematic")?.0.center() - egui::vec2(0.0, 30.0);
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, opener);
    assert!(egui::Popup::is_id_open(&ctx, workspace::popup_id()));
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let planned = ribbon_text(&output, "Simulate")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, planned);
    assert!(
        egui::Popup::is_id_open(&ctx, workspace::popup_id()),
        "Planned rows cannot dismiss or activate"
    );
    workspace_key(&mut app, &ctx, egui::Key::S);
    assert!(!app.palette, "Menu keys must not invoke editor shortcuts");
    workspace_key(&mut app, &ctx, egui::Key::Escape);
    assert!(!egui::Popup::is_id_open(&ctx, workspace::popup_id()));
    assert!(
        app.tool == Tool::Place(PartKind::Resistor),
        "Escape dismisses the picker first"
    );
    workspace_click(&mut app, &ctx, opener);
    workspace_click(&mut app, &ctx, egui::pos2(600.0, 550.0));
    assert!(!egui::Popup::is_id_open(&ctx, workspace::popup_id()));
    assert_eq!(
        app.document(),
        &document,
        "Dismissal must not place a symbol behind the popup"
    );
    workspace_click(&mut app, &ctx, opener);
    workspace_key(&mut app, &ctx, egui::Key::ArrowDown);
    workspace_key(&mut app, &ctx, egui::Key::Enter);
    assert!(
        !egui::Popup::is_id_open(&ctx, workspace::popup_id()),
        "Keyboard activation selects Current"
    );
    assert_eq!(app.document(), &document);
    assert_eq!(app.session.revision(), revision);
    assert_eq!(app.selected_symbol(), selected);
    assert!(!app.session.dirty());
    Ok(())
}

#[test]
fn expanded_workspace_cards_keep_reference_geometry_and_readable_type() -> Result<()> {
    for scale in [1.0, 1.25] {
        let mut app = App::create(Options {
            demo: true,
            capture: None,
            exit_after_capture: false,
            capture_workspace: false,
            capture_library: false,
            capture_panels: false,
            capture_search: false,
            capture_properties: false,
            capture_nets: false,
            capture_wire_menu: false,
            capture_text: false,
            capture_selection: false,
            open: None,
        })?;
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        let mut t = theme::Theme::default();
        t.metrics.body *= scale;
        t.metrics.small *= scale;
        t.metrics.heading *= scale;
        t.metrics.control_height *= scale;
        theme::set(&ctx, t);
        egui::Popup::open_id(&ctx, workspace::popup_id());
        for _ in 0..4 {
            workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
        }
        let output = workspace_frame(&mut app, &ctx, vec![]);
        for label in [
            "WORKSPACE",
            "Current",
            "PCB layout",
            "3D render",
            "Simulate",
            "Set up circuit models, analyses and waveform probes",
            "Capture circuits, place symbols and connect pins",
            "Capture · Analyze · Libraries · Manage · View",
            "Place components, route copper and define the board",
            "Inspect the assembled board, layers and components",
        ] {
            let (text, clip) = ribbon_text(&output, label)?;
            assert!(
                clip.contains_rect(text),
                "Clipped workspace content: {label}"
            );
        }
        let titles: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "PCB layout" => Some(text),
                _ => None,
            })
            .collect();
        assert_eq!(
            titles.len(),
            1,
            "Titles must not be overdrawn to simulate font weight"
        );
        assert_eq!(
            titles[0].galley.job.sections[0].format.font_id,
            theme::semibold(t.metrics.heading)
        );
        assert!(
            output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::Shape::Rect(rect) if (rect.rect.width() - 450.0 * scale).abs() < 1.0
                    && (rect.rect.height() - 72.0 * scale).abs() < 1.0
            )),
            "Current card retains the source component's proportions"
        );
        let current = ribbon_text(&output, "Current")?.0;
        let layout = ribbon_text(&output, "PCB layout")?.0;
        let render = ribbon_text(&output, "3D render")?.0;
        assert!(current.bottom() < layout.top() && layout.bottom() < render.top());
        output.drop_without_applying_deltas();
    }
    Ok(())
}

#[test]
fn theme_publication_is_local_and_does_not_change_the_electronics_document() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let before = app.document().clone();
    let revision = app.session.revision();
    let first = egui::Context::default();
    let second = egui::Context::default();
    theme::apply(&first);
    theme::apply(&second);
    let mut alternate = theme::Theme::default();
    alternate.ui.surface = egui::Color32::from_rgb(14, 18, 22);
    alternate.canvas.background = egui::Color32::from_rgb(8, 12, 16);
    alternate.icons.line = egui::Color32::from_rgb(255, 220, 160);
    theme::set(&first, alternate);
    assert_eq!(
        first.style_of(alternate.base).visuals.panel_fill,
        alternate.ui.surface
    );
    assert_eq!(
        theme::current(&first).canvas.background,
        alternate.canvas.background
    );
    assert_eq!(theme::current(&first).icons, alternate.icons);
    assert_ne!(theme::current(&second).ui.surface, alternate.ui.surface);
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1600.0, 1000.0),
        )),
        ..Default::default()
    };
    first
        .run_ui(input, |ui| app.show(ui))
        .drop_without_applying_deltas();
    assert_eq!(app.document(), &before);
    assert_eq!(app.session.revision(), revision);
    assert!(!app.session.dirty());
    Ok(())
}

fn finish_file_job(app: &mut App, ctx: &egui::Context) {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while app.busy() && std::time::Instant::now() < deadline {
        app.poll(ctx);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!app.busy(), "Bounded file fixture did not complete");
}

#[test]
fn empty_workspace_display_controls_are_visible() -> Result<()> {
    let mut app = App::create(Options {
        demo: false,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    let mut visible = false;
    for index in 0..20 {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600.0, 1000.0),
            )),
            time: Some(index as f64 * 0.05),
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| app.show(ui));
        visible = output.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Symbols" => {
                shape.clip_rect.contains(text.pos)
            }
            _ => false,
        });
        output.drop_without_applying_deltas();
    }
    assert!(
        visible,
        "Selection controls must be visible in an empty workspace"
    );
    Ok(())
}

#[test]
fn desktop_save_reopen_and_failed_open_keep_session_contract() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("desktop.pc");
    app.disk_bytes = Some(persistence::write(&path, app.document(), None)?);
    app.path = Some(path.clone());
    let symbol = app.document().symbols.first().ok_or("No symbol")?.id;
    app.command(Command::Edit {
        symbol,
        reference: "J9".into(),
        value: "Bench input".into(),
    });
    assert!(app.session.dirty());
    let saved = app.document().clone();
    app.save(false);
    finish_file_job(&mut app, &ctx);
    assert!(!app.session.dirty());
    assert_eq!(persistence::read(&path)?.0, saved);
    let old_incarnation = app.session.incarnation;
    app.load(Some(path));
    finish_file_job(&mut app, &ctx);
    assert_eq!(app.document(), &saved);
    assert_ne!(app.session.incarnation, old_incarnation);
    assert!(!app.session.can_undo());
    app.load(Some(temp.path().join("missing.pc")));
    finish_file_job(&mut app, &ctx);
    assert_eq!(app.document(), &saved);
    assert_eq!(app.drawer, Some("Messages"));
    Ok(())
}

#[test]
fn file_dialog_blocks_edits_and_pending_capture_survives_frames() -> Result<()> {
    let mut app = App::create(Options {
        demo: true,
        capture: Some("unused-test-capture.png".into()),
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    let before = app.document().clone();
    let (tx, rx) = mpsc::channel();
    app.job = Some(rx);
    app.command(Command::Place {
        kind: PartKind::Led,
        at: Point::default(),
    });
    app.history(false);
    assert_eq!(app.document(), &before);
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1600.0, 1000.0),
        )),
        ..Default::default()
    };
    ctx.run_ui(input, |ui| app.show(ui))
        .drop_without_applying_deltas();
    assert!(
        app.options.capture.is_some(),
        "Capture path must remain until screenshot event arrives"
    );
    tx.send(FileResult::Error("Original test failure".into()))
        .map_err(|e| e.to_string())?;
    app.poll(&ctx);
    assert_eq!(app.document(), &before);
    assert!(!app.busy());
    assert_eq!(app.drawer, Some("Messages"));
    Ok(())
}

#[test]
fn dirty_project_requires_resolution_before_new_document() -> Result<()> {
    let mut app = App::create(Options {
        demo: false,
        capture: None,
        exit_after_capture: false,
        capture_workspace: false,
        capture_library: false,
        capture_panels: false,
        capture_search: false,
        capture_properties: false,
        capture_nets: false,
        capture_wire_menu: false,
        capture_text: false,
        capture_selection: false,
        open: None,
    })?;
    app.command(Command::Place {
        kind: PartKind::Resistor,
        at: Point::default(),
    });
    let before = app.document().clone();
    app.request(Action::New);
    assert!(matches!(app.pending, Some(Action::New)));
    app.command(Command::Delete {
        symbol: before.symbols.first().ok_or("No symbol")?.id,
    });
    assert_eq!(app.document(), &before);
    app.pending = None; // The cancel button resolves the dialog without mutation.
    assert_eq!(app.document(), &before);
    Ok(())
}

#[test]
fn ribbon_tabs_switch_command_sets_without_opening_panels() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    let before = app.document().clone();
    let selection = app.selected_symbol();
    let panels = (app.browser, app.properties, app.filters, app.drawer);
    for (label, tab, tool) in [
        ("Analyze", ribbon::Tab::Analyze, "ERC"),
        ("Libraries", ribbon::Tab::Libraries, "Parts"),
        ("Manage", ribbon::Tab::Manage, "Save"),
        ("View", ribbon::Tab::View, "Fit page"),
        ("Capture", ribbon::Tab::Capture, "Symbol"),
    ] {
        let output = workspace_frame(&mut app, &ctx, vec![]);
        let p = ribbon_text(&output, label)?.0.center();
        output.drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, p);
        assert_eq!(app.ribbon_tab, tab);
        assert!(!app.library && !app.palette && !egui::Popup::is_any_open(&ctx));
        assert_eq!(
            (app.browser, app.properties, app.filters, app.drawer),
            panels
        );
        assert_eq!(app.selected_symbol(), selection);
        assert_eq!(app.document(), &before);
        let output = workspace_frame(&mut app, &ctx, vec![]);
        assert!(
            ribbon_text(&output, tool).is_ok(),
            "Missing {tool} after {label}"
        );
        if tab != ribbon::Tab::Capture {
            assert!(ribbon_text(&output, "Rotate").is_err());
        }
        output.drop_without_applying_deltas();
    }
    // A command in the active tab, unlike the tab itself, opens the library.
    app.ribbon_tab = ribbon::Tab::Libraries;
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let p = ribbon_text(&output, "Parts")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, p);
    assert!(app.library);
    Ok(())
}

#[test]
fn browser_lists_expand_select_and_keep_row_geometry_without_opening_drawer() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.panel_ui.symbols_open.insert(app.active_sheet());
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let picked = app.selection.clone();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let at = panel_text(&output, app.layout.browser, "Symbols")?
        .0
        .center();
    let (row, font) = panel_text(&output, app.layout.browser, "J1  12 V input")?;
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, row.center());
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let (after, bold) = panel_text(&output, app.layout.browser, "J1  12 V input")?;
    output.drop_without_applying_deltas();
    assert_eq!(row.min, after.min);
    assert_eq!(row.height(), after.height());
    assert_ne!(font, bold);
    assert_ne!(app.selection, picked);
    workspace_click(&mut app, &ctx, at);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let nets = panel_text(&output, app.layout.browser, "Nets")?.0.center();
    let hidden = panel_text(&output, app.layout.browser, "J1  12 V input").is_err();
    output.drop_without_applying_deltas();
    assert!(hidden);
    workspace_click(&mut app, &ctx, nets);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let rail = panel_text(&output, app.layout.browser, "+12V")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, rail);
    assert_eq!(
        app.nets.selected_net().and_then(|n| n.name.as_deref()),
        Some("+12V")
    );
    assert!(app.drawer.is_none());
    assert_eq!(app.document(), &before);
    Ok(())
}

pub(super) fn panel_text(
    output: &egui::FullOutput,
    panel: egui::Rect,
    label: &str,
) -> Result<(egui::Rect, egui::FontFamily)> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(t) if t.galley.job.text == label => {
                let rect = t.galley.rect.translate(t.pos.to_vec2());
                (panel.contains_rect(rect) && shape.clip_rect.contains_rect(rect))
                    .then(|| (rect, t.galley.job.sections[0].format.font_id.family.clone()))
            }
            _ => None,
        })
        .ok_or_else(|| format!("Missing visible panel text {label}").into())
}

#[test]
fn properties_can_float_dock_close_and_reopen_without_editing() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let origin = measured_area(&ctx, "properties")?.min + egui::vec2(75.0, 24.0);
    let target = egui::pos2(480.0, 280.0);
    workspace_frame(
        &mut app,
        &ctx,
        vec![
            egui::Event::PointerMoved(origin),
            egui::Event::PointerButton {
                pos: origin,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    )
    .drop_without_applying_deltas();
    workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(target)])
        .drop_without_applying_deltas();
    release_library_drag(&mut app, &ctx, target);
    assert_eq!(app.inspector.dock, library::Dock::Floating);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let floating = measured_area(&ctx, "properties")?;
    assert!(floating.left() < 600.0);
    assert!(app.layout.bounds.contains_rect(floating));
    let close = egui::pos2(floating.right() - 18.0, floating.top() + 24.0);
    workspace_click(&mut app, &ctx, close);
    assert!(!app.properties);
    app.ribbon_tab = ribbon::Tab::View;
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let p = ribbon_text(&output, "Properties")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, p);
    assert!(app.properties);
    app.inspector.dock = library::Dock::Left;
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    assert!(!app.layout.browser.intersects(app.layout.properties));
    assert!(app.layout.bounds.contains_rect(app.layout.properties));
    app.open_properties();
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let found = ribbon_text(&output, "U1 · Symbol properties").is_ok();
    output.drop_without_applying_deltas();
    assert!(found);
    workspace_key(&mut app, &ctx, egui::Key::Escape);
    assert!(app.inspector.draft.is_none());
    assert_eq!(app.document(), &before);
    Ok(())
}

#[test]
fn browser_scroll_reaches_last_row_and_height_is_user_resizable() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.panel_ui.symbols_open.insert(app.active_sheet());
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    app.panel_ui.browser_height = Some(210.0);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let body = app.panel_ui.browser_body.ok_or("No browser body")?;
    for _ in 0..8 {
        workspace_frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(body.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -1200.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let visible = panel_text(&output, app.layout.browser, "Nets").is_ok();
    output.drop_without_applying_deltas();
    assert!(visible, "Last row must be fully reachable");
    let old = app.layout.browser;
    let grip = app.panel_ui.browser_grip.ok_or("No resize grip")?.center();
    let target = grip - egui::vec2(0.0, 90.0);
    workspace_frame(
        &mut app,
        &ctx,
        vec![
            egui::Event::PointerMoved(grip),
            egui::Event::PointerButton {
                pos: grip,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    )
    .drop_without_applying_deltas();
    workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(target)])
        .drop_without_applying_deltas();
    release_library_drag(&mut app, &ctx, target);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    assert!(
        app.layout.browser.height() < old.height() - 40.0,
        "Dragging must actually resize the card"
    );
    assert!(app.layout.bounds.contains_rect(app.layout.browser));
    assert_eq!(app.document(), &before);
    Ok(())
}

#[test]
fn net_browser_selects_and_names_real_connectivity_with_undo() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.open_sheet_index(true);
    app.panel_ui.search = "+3V3".into();
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let net = app
        .nets
        .snapshot
        .nets
        .iter()
        .find(|n| n.name.as_deref() == Some("+3V3"))
        .cloned()
        .ok_or("Missing supply net")?;
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let row = panel_text(
        &output,
        app.panel_ui.search_body.ok_or("No result body")?,
        "+3V3",
    )?
    .0
    .center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, row);
    assert_eq!(app.nets.selected.as_ref(), Some(&net.anchor));
    assert!(app.selected_symbol().is_none());
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    assert!(!app.layout.properties.intersects(app.layout.filters));
    let before = app.document().clone();
    app.nets.name = "LOGIC_3V3".into();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let apply = ribbon_text(&output, "Apply name")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, apply);
    assert_eq!(
        app.nets.selected_net().and_then(|n| n.name.as_deref()),
        Some("LOGIC_3V3")
    );
    app.history(false);
    assert_eq!(app.document(), &before);
    assert!(app.nets.selected.is_none());
    app.open_sheet_index(true);
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let p = ribbon_text(&output, "Netlist…")?.0.center();
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, p);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let visible = ribbon_text(&output, "Netlist representation").is_ok();
    output.drop_without_applying_deltas();
    assert!(visible);
    workspace_key(&mut app, &ctx, egui::Key::Escape);
    assert!(!app.nets.modal());
    assert_eq!(app.document(), &before);
    Ok(())
}

#[test]
fn canvas_context_menu_stays_open_and_deletes_the_clicked_wire() -> Result<()> {
    use pcbcraft_foundation::wiring::Anchor;
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.session = Session::new(Document::empty("Context menu")?)?;
    app.canvas = Default::default();
    app.canvas.fit = false;
    app.selection.clear();
    app.apply(
        0,
        Command::DrawWire {
            from: Anchor::Free(Point {
                x: -15 * GRID_NM,
                y: 0,
            }),
            to: Anchor::Free(Point {
                x: 15 * GRID_NM,
                y: 0,
            }),
            bends: vec![],
        },
    )?;
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let full = app.layout.bounds;
    let canvas = egui::Rect::from_min_max(
        full.min + egui::vec2(284.0, 52.0),
        full.max - egui::vec2(288.0, 85.0),
    );
    let at = app.canvas.screen(Point::default(), canvas);
    for pressed in [true, false] {
        workspace_frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Secondary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
    }
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let delete = ribbon_text(&output, "Delete wire")?.0.center();
    assert!(ribbon_text(&output, "Start wire here").is_ok());
    output.drop_without_applying_deltas();
    workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(delete)])
        .drop_without_applying_deltas();
    let output = workspace_frame(&mut app, &ctx, vec![]);
    assert!(
        ribbon_text(&output, "Delete wire").is_ok(),
        "Moving into the popup must not lose its target"
    );
    output.drop_without_applying_deltas();
    assert_eq!(*app.document(), before);
    workspace_click(&mut app, &ctx, delete);
    assert!(app.document().wires.is_empty());
    app.history(false);
    assert_eq!(*app.document(), before);
    Ok(())
}

#[test]
fn file_menu_and_filter_labels_fit_at_compact_and_scaled_sizes() -> Result<()> {
    for (width, scale) in [(1000.0, 1.0), (1600.0, 1.3)] {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        app.open_sheet_index(true);
        let mut presentation = theme::current(&ctx);
        presentation.metrics.body *= scale;
        presentation.metrics.small *= scale;
        theme::set(&ctx, presentation);
        let frame = |app: &mut App, events| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 1000.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| app.show(ui),
            )
        };
        for _ in 0..4 {
            frame(&mut app, vec![]).drop_without_applying_deltas();
        }
        let output = frame(&mut app, vec![]);
        for label in ["Symbols", "Pins", "Wires", "Other items", "Rule areas"] {
            // Locate the filter text even if another tab has the same label.
            let fits = output.shapes.iter().any(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == label && t.pos.x > width - 300.0 => s
                    .clip_rect
                    .expand(0.5)
                    .contains_rect(t.galley.rect.translate(t.pos.to_vec2())),
                _ => false,
            });
            assert!(
                fits,
                "Filter label {label} must be fully visible at {width}/{scale}"
            );
        }
        {
            let label = "Netlist…";
            let (r, clip) = ribbon_text(&output, label)?;
            assert!(
                clip.contains_rect(r),
                "Net toolbar action {label} must not be clipped"
            );
        }
        let file = ribbon_text(&output, "File")?.0.center();
        output.drop_without_applying_deltas();
        let before = app.document().clone();
        for pressed in [true, false] {
            frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(file),
                    egui::Event::PointerButton {
                        pos: file,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .drop_without_applying_deltas();
        }
        for _ in 0..3 {
            frame(&mut app, vec![]).drop_without_applying_deltas();
        }
        let output = frame(&mut app, vec![]);
        assert_eq!(
            *app.document(),
            before,
            "Opening a menu cannot execute its first action"
        );
        let (new, new_clip) = ribbon_text(&output, "New schematic")?;
        let (open, open_clip) = ribbon_text(&output, "Open draft…")?;
        let (key, key_clip) = ribbon_text(&output, "Ctrl+N")?;
        assert!((new.left() - open.left()).abs() < 0.5);
        assert!(
            key.left() > new.right() + 4.0,
            "Shortcut has its own column"
        );
        assert!(
            new_clip.contains_rect(new)
                && open_clip.contains_rect(open)
                && key_clip.contains_rect(key)
        );
        let choose = new.center();
        output.drop_without_applying_deltas();
        for pressed in [true, false] {
            frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(choose),
                    egui::Event::PointerButton {
                        pos: choose,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .drop_without_applying_deltas();
        }
        assert!(
            app.document().symbols.is_empty(),
            "Shared menu still dispatches New"
        );
    }
    Ok(())
}

#[test]
fn browser_disclosures_do_not_navigate_or_edit_and_sheet_actions_are_honest() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    let before = app.document().clone();
    let selection = app.selection.clone();
    let revision = app.session.revision();
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let main = panel_text(&output, app.layout.browser, "Main")?.0;
    output.drop_without_applying_deltas();
    let fold = egui::pos2(main.left() - 34.0, main.center().y);
    workspace_click(&mut app, &ctx, fold);
    assert!(app.panel_ui.main_folded);
    let output = workspace_frame(&mut app, &ctx, vec![]);
    assert!(panel_text(&output, app.layout.browser, "Symbols").is_err());
    assert!(panel_text(&output, app.layout.browser, "Main").is_ok());
    output.drop_without_applying_deltas();
    assert_eq!(app.selection, selection);
    assert!(app.drawer.is_none());
    workspace_click(&mut app, &ctx, fold);
    assert!(!app.panel_ui.main_folded);
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let sheet = panel_text(&output, app.layout.browser, "Sheets")?
        .0
        .center();
    output.drop_without_applying_deltas();
    for pressed in [true, false] {
        workspace_frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(sheet),
                egui::Event::PointerButton {
                    pos: sheet,
                    button: egui::PointerButton::Secondary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
    }
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let new = ribbon_text(&output, "New sheet…")?.0.center();
    assert!(ribbon_text(&output, "New subsheet…").is_ok());
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, new);
    assert!(
        app.sheets.dialog.is_some(),
        "Sheet creation opens a real reviewed operation"
    );
    assert_eq!(app.document(), &before);
    assert_eq!(app.session.revision(), revision);
    assert_eq!(app.selection, selection);
    Ok(())
}

#[test]
fn sheet_results_scroll_to_last_symbol_and_browser_remains_visible() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.panel_ui.symbols_open.remove(&app.active_sheet());
    app.open_sheet_index(false);
    for _ in 0..4 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let body = app.panel_ui.search_body.ok_or("No search body")?;
    for _ in 0..8 {
        workspace_frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(body.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -1200.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .drop_without_applying_deltas();
    }
    let output = workspace_frame(&mut app, &ctx, vec![]);
    let last = panel_text(&output, body, "D2")?.0.center();
    for label in ["Main", "Symbols", "Nets"] {
        assert!(
            panel_text(&output, app.layout.browser, label).is_ok(),
            "Tree row clipped: {label}"
        );
    }
    output.drop_without_applying_deltas();
    workspace_click(&mut app, &ctx, last);
    assert_eq!(
        app.selected_symbol()
            .and_then(|id| app.document().component(id))
            .map(|c| c.reference.as_str()),
        Some("D2")
    );
    assert_eq!(app.document(), &before);
    Ok(())
}

#[test]
fn demo_designators_do_not_intersect_authored_wires() -> Result<()> {
    use pcbcraft_foundation::text::TextTarget;
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    for size in [egui::vec2(1000.0, 700.0), egui::vec2(1600.0, 1000.0)] {
        app.canvas.fit = true;
        for _ in 0..3 {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| app.show(ui),
            )
            .drop_without_applying_deltas();
        }
        let mut collisions = Vec::new();
        let doc = app.document();
        for (target, rect) in &app.canvas.text_hits {
            if !matches!(
                target,
                TextTarget::Reference(_) | TextTarget::Value(_) | TextTarget::Note(_)
            ) {
                continue;
            }
            for wire in doc.wires.iter().filter(|w| w.draw_path) {
                let a = doc.endpoint(&wire.from).ok_or("endpoint")?;
                let b = doc.endpoint(&wire.to).ok_or("endpoint")?;
                let path: Vec<_> = std::iter::once(a)
                    .chain(wire.bends.iter().flatten().copied())
                    .chain(std::iter::once(b))
                    .map(|p| {
                        app.canvas.screen(
                            p,
                            egui::Rect::from_min_max(
                                app.layout.bounds.min + egui::vec2(272.0, 40.0),
                                app.layout.bounds.max - egui::vec2(276.0, 73.0),
                            ),
                        )
                    })
                    .collect();
                for pair in path.windows(2) {
                    if rect
                        .shrink(0.5)
                        .intersects(egui::Rect::from_two_pos(pair[0], pair[1]).expand(0.3))
                    {
                        collisions.push(format!(
                            "{:?} {} at {:?}",
                            target,
                            target.content(doc).unwrap_or_default(),
                            rect
                        ));
                    }
                }
            }
        }
        assert!(collisions.is_empty(), "{}", collisions.join("\n"));
    }
    Ok(())
}

#[test]
fn drawer_resizes_vertically_retains_height_and_does_not_edit_or_reframe() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.drawer = Some("Search");
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    let before = app.document().clone();
    let camera = (app.canvas.zoom, app.canvas.offset);
    let old_height = app.layout.drawer.height();
    let grip = app
        .panel_ui
        .drawer_grip
        .ok_or("Missing drawer handle")?
        .center();
    let target = grip - egui::vec2(0.0, 110.0);
    workspace_frame(
        &mut app,
        &ctx,
        vec![
            egui::Event::PointerMoved(grip),
            egui::Event::PointerButton {
                pos: grip,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    )
    .drop_without_applying_deltas();
    workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(target)])
        .drop_without_applying_deltas();
    release_library_drag(&mut app, &ctx, target);
    for _ in 0..3 {
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    }
    assert!(app.layout.drawer.height() > old_height + 65.0);
    assert!(app.layout.bounds.contains_rect(app.layout.drawer));
    assert_eq!(camera, (app.canvas.zoom, app.canvas.offset));
    let height = app.layout.drawer.height();
    app.drawer = None;
    workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    app.drawer = Some("Messages");
    workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    assert_eq!(height, app.layout.drawer.height());
    app.panel_ui.drawer_height = Some(100_000.0);
    workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    assert!(app.layout.bounds.contains_rect(app.layout.drawer));
    assert_eq!(before, *app.document());
    Ok(())
}

#[test]
fn demo_full_size_text_has_clearance_from_other_labels() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    app.canvas.fit = true;
    for _ in 0..4 {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 1000.0),
                )),
                ..Default::default()
            },
            |ui| app.show(ui),
        )
        .drop_without_applying_deltas();
    }
    let mut collisions = Vec::new();
    for (i, (target, r)) in app.canvas.text_hits.iter().enumerate() {
        for (other, s) in app.canvas.text_hits.iter().skip(i + 1) {
            if r.shrink(0.7).intersects(s.shrink(0.7)) {
                collisions.push(format!(
                    "{target:?} {} {r:?} / {other:?} {} {s:?}",
                    target.content(app.document()).unwrap_or_default(),
                    other.content(app.document()).unwrap_or_default()
                ));
            }
        }
    }
    assert!(collisions.is_empty(), "{}", collisions.join("\n"));
    Ok(())
}

#[test]
fn symbol_placement_repeats_and_secondary_click_only_cancels() -> Result<()> {
    let (mut app, ctx) = library_fixture()?;
    app.library = false;
    workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    let center = app.layout.bounds.center();
    let n = app.document().symbols.len();
    let before = app.session.project().clone();
    app.tool = Tool::Place(PartKind::Resistor);
    workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(center)])
        .drop_without_applying_deltas();
    assert_eq!(app.session.project(), &before);
    workspace_click(&mut app, &ctx, center);
    workspace_click(&mut app, &ctx, center + egui::vec2(45.0, 45.0));
    assert_eq!(app.document().symbols.len(), n + 2);
    assert_eq!(app.tool, Tool::Place(PartKind::Resistor));
    let revision = app.session.revision();
    for pressed in [true, false] {
        workspace_frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        )
        .drop_without_applying_deltas();
    }
    assert_eq!(app.tool, Tool::Select);
    assert_eq!(app.session.revision(), revision);
    assert!(!egui::Popup::is_any_open(&ctx));
    app.history(false);
    assert_eq!(app.document().symbols.len(), n + 1);
    Ok(())
}
