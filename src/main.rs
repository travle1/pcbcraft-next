#![forbid(unsafe_code)]
mod desktop;

fn main() -> eframe::Result {
    let mut options = desktop::Options {
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
        open: None,
    };
    let mut compact = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => options.demo = true,
            "--capture" => options.capture = args.next().map(Into::into),
            "--exit-after-capture" => options.exit_after_capture = true,
            "--capture-workspace" => options.capture_workspace = true,
            "--capture-library" => options.capture_library = true,
            "--capture-panels" => options.capture_panels = true,
            "--capture-search" => options.capture_search = true,
            "--capture-properties" => options.capture_properties = true,
            "--capture-nets" => options.capture_nets = true,
            "--capture-wire-menu" => options.capture_wire_menu = true,
            "--capture-compact" => compact = true,
            "--help" | "-h" => {
                println!(
                    "PCBcraft — standalone schematic draft\nUsage: pcbcraft [project.pc] [--demo] [--capture image.png] [--capture-workspace | --capture-library | --capture-panels | --capture-search | --capture-properties | --capture-nets | --capture-wire-menu] [--capture-compact] [--exit-after-capture]"
                );
                return Ok(());
            }
            _ if !arg.starts_with('-') => options.open = Some(arg.into()),
            _ => {
                eprintln!("Unknown option: {arg}");
                return Ok(());
            }
        }
    }
    let capture = options.capture.is_some();
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PCBcraft")
            .with_active(!capture)
            .with_app_id("pcbcraft")
            .with_inner_size(if compact {
                [1000.0, 700.0]
            } else {
                [1600.0, 1000.0]
            })
            .with_min_inner_size([1000.0, 700.0])
            .with_decorations(false)
            .with_icon(eframe::egui::IconData::default()),
        ..Default::default()
    };
    eframe::run_native(
        "PCBcraft",
        native,
        Box::new(move |cc| {
            desktop::App::new(cc, options)
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(Into::into)
        }),
    )
}
