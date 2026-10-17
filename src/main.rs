//! Desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod notices;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use wren::app::App;
use wren::backend::Waker;
use wren::paths::AppDirs;
use wren::settings::Settings;

/// A fast, native client for Microsoft Teams.
#[derive(Debug, Parser)]
#[command(name = "wren", version, about)]
struct Cli {
    /// Print the bundled license texts and third-party notices, then quit.
    #[arg(long)]
    licenses: bool,
    /// Start with offline sample data instead of an account.
    #[arg(long)]
    demo: bool,
    /// Log more.
    #[arg(short, long)]
    verbose: bool,
    /// Start in the tray without a window (when "keep running" is on).
    #[arg(long)]
    start_hidden: bool,
    /// Save the window as a PNG after `--screenshot-delay` and quit.
    /// Implies `--demo`.
    #[arg(long, value_name = "PATH")]
    screenshot: Option<PathBuf>,
    /// Milliseconds to wait before the screenshot.
    #[arg(long, value_name = "MS", default_value_t = 1500)]
    screenshot_delay: u64,
    /// Window size as WxH points, for screenshots.
    #[arg(long, value_name = "WxH")]
    size: Option<String>,
    /// Demo view: `chats`, `channel`, `calendar`, `calls`, `signin`,
    /// `settings`, `switcher`, `shortcuts`, plus `light` or `dark`, joined
    /// with commas.
    #[arg(long)]
    page: Option<String>,
    /// Print the time to the first frame and quit.
    #[arg(long)]
    measure_startup: bool,
}

fn main() -> eframe::Result<()> {
    let started = Instant::now();
    let launch = fastframe_update::intercept(&wren::updates::CONFIG);
    let cli = Cli::parse_from(&launch.arguments);
    if cli.licenses {
        print!("{}", notices::TEXT);
        return Ok(());
    }
    let demo = cli.demo || cli.screenshot.is_some() || cli.page.is_some();
    let waker = Waker::default();

    // One running copy; a second launch shows the first one's window.
    let (instance, requests) = if demo || cli.measure_startup {
        (None, None)
    } else {
        match wren::platform::single_instance(&waker) {
            Some((guard, requests)) => (guard, Some(requests)),
            None => {
                eprintln!("Wren is already running; asked it to show its window.");
                return Ok(());
            }
        }
    };

    let dirs = if demo {
        AppDirs::under(&std::env::temp_dir().join(format!("wren-demo-{}", std::process::id())))
    } else {
        AppDirs::discover()
    };
    dirs.ensure()
        .map_err(|e| eframe::Error::AppCreation(e.into()))?;
    let filter = if cli.verbose {
        "info,wren=debug"
    } else {
        "warn,wren=info"
    };
    let logging = fastframe_log::Logging::new("wren", env!("CARGO_PKG_VERSION"))
        .filter(filter)
        .panic_log(dirs.panic_log());
    let logging = if demo {
        logging
    } else {
        logging.file(dirs.log_file())
    };
    logging
        .init()
        .map_err(|e| eframe::Error::AppCreation(e.into()))?;

    let settings = if demo {
        Settings::default()
    } else {
        Settings::load(&dirs.settings_file())
    };
    let size = parse_size(cli.size.as_deref());
    let geometry = settings.window;
    let mut app = App::new(dirs.clone(), settings, waker.clone(), demo);
    app.service = wren::backend::Backend::spawn(dirs.clone(), waker.clone());
    if demo {
        // Native demo windows may paste images too. Headless tests retain
        // their detached backend; this worker is never signed in.
        app.accounts[0].backend = wren::backend::Backend::spawn(dirs.clone(), waker.clone());
    }
    app.update_receipt = launch.receipt;
    if let Some(error) = launch.error {
        app.toast(wren::model::ToastKind::Error, error);
    }
    app.requests = requests;
    app.perf.started = Some(started);
    log::debug!(
        "startup: state ready at {} ms",
        started.elapsed().as_millis()
    );
    if let Some(page) = &cli.page {
        apply_page(&mut app, page);
    }
    if !demo && app.settings.keep_running {
        app.tray = wren::platform::tray(&waker);
    }
    let start_hidden = cli.start_hidden && app.tray.as_ref().is_some_and(|t| t.is_shown());

    let screenshot = cli.screenshot.clone().map(|path| Shot {
        path,
        due: Instant::now() + Duration::from_millis(cli.screenshot_delay),
        asked: false,
    });
    let reproducible = screenshot.is_some();
    let measure = cli.measure_startup;
    let persistence = (!demo).then(|| dirs.state.join("window.ron"));

    fastframe_shell::Shell::new(app, &waker)
        .start_hidden(start_hidden)
        .idle(fastframe_tray::idle)
        .run(|lease| {
            let screenshot = screenshot.clone();
            let mut viewport = egui::ViewportBuilder::default()
                .with_title(if demo { "Wren (demo)" } else { "Wren" })
                .with_app_id("wren")
                .with_inner_size(
                    size.or(geometry.map(|g| [g.width, g.height]))
                        .unwrap_or([1240.0, 800.0]),
                )
                // A smaller --size (screenshots of the docked panel) lowers
                // the minimum to match.
                .with_min_inner_size(
                    size.map_or([720.0, 480.0], |[w, h]| [w.min(720.0), h.min(480.0)]),
                )
                .with_icon(wren::platform::window_icon())
                // macOS: Wren draws its own title bar under the traffic lights.
                .with_fullsize_content_view(true)
                .with_titlebar_shown(false)
                .with_title_shown(false);
            if size.is_none()
                && let Some(g) = geometry
                && let (Some(x), Some(y)) = (g.x, g.y)
            {
                viewport = viewport.with_position([x, y]);
            }
            let options = eframe::NativeOptions {
                viewport,
                persist_window: !demo,
                persistence_path: persistence.clone(),
                ..Default::default()
            };
            eframe::run_native(
                "Wren",
                options,
                Box::new(move |cc| {
                    log::debug!(
                        "startup: window ready at {} ms",
                        started.elapsed().as_millis()
                    );
                    wren::ui::install(&cc.egui_ctx, reproducible);
                    log::debug!(
                        "startup: fonts and loaders ready at {} ms",
                        started.elapsed().as_millis()
                    );
                    let mut app = lease.take(&cc.egui_ctx);
                    if let Some(tray) = &mut app.tray {
                        tray.attach();
                    }
                    app.window_hidden = false;
                    Ok(Box::new(Window {
                        pacing: wren::pacing::FramePacer::new(cc),
                        app,
                        recovery_checked: false,
                        paste_input: wren::clipboard::PasteInput::default(),
                        scrolling: fastframe_scroll::Scrolling::default(),
                        screenshot,
                        measure,
                    }))
                }),
            )
        })?;
    drop(instance);
    Ok(())
}

fn parse_size(value: Option<&str>) -> Option<[f32; 2]> {
    let (w, h) = value?.split_once('x')?;
    Some([w.trim().parse().ok()?, h.trim().parse().ok()?])
}

/// Sets up a demo view for screenshots and design work.
fn apply_page(app: &mut App, page: &str) {
    use wren::model::{Action, ConversationId, Section};
    use wren::settings::ThemeChoice;
    for part in page.split(',').map(str::trim) {
        match part {
            "light" => app.settings.theme = ThemeChoice::Light,
            "dark" => app.settings.theme = ThemeChoice::Dark,
            "chats" => {}
            "channel" => {
                app.actions
                    .push(Action::OpenConversation(ConversationId::Channel {
                        team: "t-eng".into(),
                        channel: "19:eng-desktop@thread.tacv2".into(),
                    }));
            }
            "group" => {
                app.actions
                    .push(Action::OpenConversation(ConversationId::Chat(
                        "19:demo-launch@thread.v2".into(),
                    )));
            }
            "calendar" => {
                app.section = Section::Calendar;
                app.open = None;
            }
            "calls" => {
                app.section = Section::Calls;
                app.open = None;
            }
            "settings" => app.actions.push(Action::OpenSettings),
            "switcher" => {
                app.actions.push(Action::OpenSwitcher);
            }
            "shortcuts" => app.actions.push(Action::OpenShortcuts),
            "perf" => app.settings.perf_overlay = true,
            "accounts" => app.open_account_menu = true,
            "newchat" => {
                app.overlay = Some(wren::app::Overlay::NewChat);
                app.new_chat.picked = app
                    .directory()
                    .into_iter()
                    .filter(|p| p.display_name == "Grace Hopper")
                    .collect();
                app.new_chat.query = "ra".into();
                app.actions.push(Action::SearchPeople("ra".into()));
            }
            "mention" => {
                let launch = ConversationId::Chat("19:demo-launch@thread.v2".into());
                app.actions.push(Action::OpenConversation(launch.clone()));
                let file = std::env::temp_dir().join("Launch checklist.pdf");
                let _ = std::fs::write(&file, vec![0u8; 182_000]);
                app.drafts.insert(
                    launch,
                    wren::app::Draft {
                        text: "Shipping checklist attached. @Mar".into(),
                        files: vec![file],
                        ..Default::default()
                    },
                );
            }
            "long-draft" => {
                if let Some(chat) = app.open.clone() {
                    let text = (0..120)
                        .map(|i| format!("SELECT column_{i} FROM example WHERE value IS NOT NULL;"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let draft = app.drafts.entry(chat).or_default();
                    draft.paste_cursor = Some(text.chars().count());
                    draft.text = text;
                    app.focus_composer = true;
                }
            }
            "images" | "image-preview" => {
                let icon =
                    image::load_from_memory(include_bytes!("../packaging/linux/wren-256.png"))
                        .expect("bundled demo image")
                        .to_rgba8();
                let mut wide = image::RgbaImage::new(3072, 256);
                for x in 0..12 {
                    image::imageops::replace(&mut wide, &icon, x * 256, 0);
                }
                let tall = image::imageops::rotate90(&wide);
                let images: Vec<_> = [&wide, &icon, &tall]
                    .into_iter()
                    .map(|pixels| {
                        wren::clipboard::encode_rgba(
                            pixels.width() as usize,
                            pixels.height() as usize,
                            pixels.as_raw(),
                        )
                        .expect("demo clipboard image")
                    })
                    .collect();
                if part == "image-preview" {
                    app.actions.push(Action::PreviewImage(images[0].uri()));
                }
                if let Some(chat) = app.open.clone() {
                    app.drafts.entry(chat).or_default().images = images;
                }
            }
            "status" => app.actions.push(Action::OpenStatusMessage),
            // A meeting joined from the calendar: docked beside the call, or
            // with the call bar along the bottom, or its QR code.
            "call" | "callbar" | "phone" => {
                if part == "call" {
                    app.pretend_docked();
                }
                if let Some(url) = app
                    .meetings
                    .iter()
                    .find(|m| m.id == "demo-design-review")
                    .and_then(|m| m.join_url.clone())
                {
                    app.actions.push(Action::JoinMeeting(url));
                }
                if part == "phone" {
                    app.actions.push(Action::JoinOnPhone);
                }
            }
            "signin" => {
                app.demo = false;
                app.chats.clear();
                app.auth = wren::backend::AuthStatus::SignedOut;
            }
            "setup" => {
                app.demo = false;
                app.chats.clear();
                app.auth = wren::backend::AuthStatus::NotConfigured;
            }
            other => eprintln!("unknown page part: {other}"),
        }
    }
}

#[derive(Clone)]
struct Shot {
    path: PathBuf,
    due: Instant,
    asked: bool,
}

/// The eframe side of one window. The [`App`] goes back to the shell when
/// the window closes.
struct Window {
    pacing: wren::pacing::FramePacer,
    app: fastframe_shell::Held<App>,
    recovery_checked: bool,
    paste_input: wren::clipboard::PasteInput,
    scrolling: fastframe_scroll::Scrolling,
    screenshot: Option<Shot>,
    measure: bool,
}

impl Window {
    fn drive_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.screenshot.as_mut() else {
            return;
        };
        ctx.request_repaint_after(Duration::from_millis(50));
        if !shot.asked && Instant::now() >= shot.due {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            shot.asked = true;
        }
        let image = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        let Some(image) = image else { return };
        let [width, height] = [image.size[0] as u32, image.size[1] as u32];
        let pixels: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|p| p.to_srgba_unmultiplied())
            .collect();
        match image::RgbaImage::from_raw(width, height, pixels)
            .map(|buffer| buffer.save(&shot.path))
        {
            Some(Ok(())) => println!("wrote {width}x{height} to {}", shot.path.display()),
            Some(Err(error)) => eprintln!("could not write {}: {error}", shot.path.display()),
            None => eprintln!("the frame buffer did not match {width}x{height}"),
        }
        self.screenshot = None;
        self.app.quit_requested = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for Window {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        self.paste_input.prepare(input);
    }
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.pacing.begin_frame(frame);
        if !std::mem::replace(&mut self.recovery_checked, true) {
            fastframe_shell::window::recover_offscreen(ctx, frame);
        }
        let app = &mut *self.app;
        app.perf.record(frame.info().cpu_usage);
        app.observe_window(ctx);
        app.background(ctx);
        fastframe_titlebar::align_traffic_lights(frame, ctx, wren::theme::HEADER_HEIGHT);
        if app.wants_show {
            app.wants_show = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if app.quit_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.drive_screenshot(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.scrolling.apply(&ctx);
        let app = &mut *self.app;
        wren::ui::show(app, ui);
        app.apply_actions(&ctx);
        if app.perf.first_frame.is_none()
            && let Some(started) = app.perf.started
        {
            let elapsed = started.elapsed();
            app.perf.first_frame = Some(elapsed);
            if let Some(receipt) = app.update_receipt.take() {
                app.service
                    .send(wren::backend::Command::AcknowledgeUpdate(receipt));
            }
            log::info!("first frame after {} ms", elapsed.as_millis());
            log::debug!(
                "window geometry: {:?}",
                ctx.input(|i| i.viewport().outer_rect)
            );
            if self.measure {
                println!("first frame after {} ms", elapsed.as_millis());
                app.quit_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        // Remember the window's size and place.
        let (size, position, maximized) = ctx.input(|i| {
            let viewport = i.viewport();
            (
                viewport.inner_rect.map(|r| r.size()),
                viewport.outer_rect.map(|r| r.min),
                viewport.maximized.unwrap_or(false),
            )
        });
        // Not while docked beside a call: the next start gets the full size.
        if let Some(size) = size
            && !app.demo
            && app.docked.is_none()
        {
            let geometry = wren::settings::WindowGeometry {
                width: size.x,
                height: size.y,
                x: position.map(|p| p.x),
                y: position.map(|p| p.y),
                maximized,
            };
            if app.settings.window != Some(geometry) && !ctx.input(|i| i.pointer.any_down()) {
                app.settings.window = Some(geometry);
                app.mark_settings_dirty();
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app.save_settings();
    }
}
