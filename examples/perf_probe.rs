//! Offline performance workloads. See docs/performance.md.
//! This executable uses demo data and never reads credentials or contacts Graph.

use std::io::{BufWriter, Write};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use egui::{Event, Modifiers, MouseWheelUnit, Pos2, Vec2};
use serde_json::json;
use wren::app::{App, Image};
use wren::backend::{Event as BackendEvent, Waker};
use wren::model::{Action, ConversationId, Section};
use wren::paths::AppDirs;
use wren::settings::{Settings, ThemeChoice};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Case {
    Idle,
    Hover,
    Scroll,
    Spinner,
    History,
    Images,
    Views,
    Unicode,
}

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum, default_value = "idle")]
    case: Case,
    #[arg(long)]
    native: bool,
    #[arg(long, default_value_t = 20)]
    seconds: u64,
    #[arg(long, default_value_t = 120)]
    frames: usize,
    #[arg(long, default_value_t = 5_000)]
    messages: usize,
    #[arg(long, default_value_t = 20)]
    images: usize,
    #[arg(long, default_value_t = 1920)]
    image_width: u32,
    #[arg(long, default_value_t = 1080)]
    image_height: u32,
    #[arg(long, default_value_t = 1.0)]
    scale: f32,
    #[arg(long)]
    light: bool,
    /// Use installed system fonts, as the real app does. Default is the CI font.
    #[arg(long)]
    system_fonts: bool,
    /// Diagnostic comparison only. This does not change Wren's defaults.
    #[arg(long)]
    reduce_texture_memory: bool,
    #[arg(long)]
    output: std::path::PathBuf,
}

struct Probe {
    args: Args,
    app: App,
    output: BufWriter<std::fs::File>,
    started: Instant,
    frames: usize,
    paints: Arc<AtomicU64>,
    pacing: Option<wren::pacing::FramePacer>,
    times: Vec<f64>,
    immediate: usize,
    causes: std::collections::BTreeMap<String, usize>,
    image: Vec<u8>,
    loaded: usize,
    current_uri: Option<String>,
    image_paints: u64,
    last_scroll: Instant,
    since_image: usize,
    done_images: bool,
    scrolling: fastframe_scroll::Scrolling,
    _dir: tempfile::TempDir,
}

impl Probe {
    fn new(args: Args, ctx: &egui::Context) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings {
            theme: if args.light {
                ThemeChoice::Light
            } else {
                ThemeChoice::Dark
            },
            notifications: false,
            ..Settings::default()
        };
        let waker = Waker::default();
        waker.attach(ctx);
        let mut app = App::new(AppDirs::under(dir.path()), settings, waker, true);
        wren::ui::install(ctx, !args.system_fonts);
        ctx.options_mut(|o| o.reduce_texture_memory = args.reduce_texture_memory);
        if matches!(args.case, Case::History | Case::Scroll) {
            let id = app.open.clone().unwrap();
            let template = app.conversation(&id).unwrap().messages[0].clone();
            app.conversations.get_mut(&id).unwrap().messages = (0..args.messages)
                .map(|i| {
                    let mut message = template.clone();
                    message.id = format!("fixture-{i}");
                    message.created = template.created - (args.messages - i) as i64 * 60_000;
                    message
                })
                .collect();
        }
        if matches!(args.case, Case::Spinner) {
            let conversation = app.open.clone().unwrap();
            app.loading
                .insert(wren::sync::Resource::Messages(conversation));
        }
        if matches!(args.case, Case::Unicode) {
            let id = app.open.clone().unwrap();
            app.conversations
                .get_mut(&id)
                .unwrap()
                .messages
                .last_mut()
                .unwrap()
                .body = wren::html::RichText::plain("日本語 中文 العربية עברית हिन्दी 🙂");
        }
        let image = if matches!(args.case, Case::Images) {
            let image = image::RgbaImage::from_fn(args.image_width, args.image_height, |x, y| {
                image::Rgba([
                    (x % 251) as u8,
                    (y % 251) as u8,
                    ((x / 32 + y / 32) % 251) as u8,
                    255,
                ])
            });
            let mut bytes = std::io::Cursor::new(Vec::new());
            image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
            bytes.into_inner()
        } else {
            Vec::new()
        };
        let output = BufWriter::new(std::fs::File::create(&args.output).unwrap());
        Self {
            args,
            app,
            output,
            started: Instant::now(),
            frames: 0,
            paints: Arc::new(AtomicU64::new(0)),
            pacing: None,
            times: Vec::new(),
            immediate: 0,
            causes: Default::default(),
            image,
            loaded: 0,
            current_uri: None,
            image_paints: 0,
            last_scroll: Instant::now(),
            since_image: 0,
            done_images: false,
            scrolling: Default::default(),
            _dir: dir,
        }
    }

    fn snapshot(&mut self, ctx: &egui::Context, stage: &str) {
        let loaders = ctx.loaders();
        let bytes: usize = loaders.bytes.lock().iter().map(|l| l.byte_size()).sum();
        let decoded: usize = loaders.image.lock().iter().map(|l| l.byte_size()).sum();
        let textures: usize = loaders.texture.lock().iter().map(|l| l.byte_size()).sum();
        let texture_manager = ctx.tex_manager();
        let manager = texture_manager.read();
        let texture_total: usize = manager.allocated().map(|(_, m)| m.bytes_used()).sum();
        let atlas = manager
            .allocated()
            .find(|(id, _)| **id == egui::TextureId::Managed(0))
            .map(|(_, m)| m.size);
        let mut times = self.times.clone();
        times.sort_by(f64::total_cmp);
        let percentile = |p: f64| {
            times
                .get(((times.len().saturating_sub(1)) as f64 * p) as usize)
                .copied()
        };
        writeln!(self.output, "{}", json!({
            "stage": stage, "case": format!("{:?}", self.args.case), "native": self.args.native,
            "seconds": self.started.elapsed().as_secs_f64(), "frames": self.frames,
            "paint_callbacks": self.paints.load(Ordering::Relaxed),
            "fallback_font_revision": wren::theme::font_revision(ctx),
            "cjk_glyph": ctx.fonts_mut(|f| f.has_glyph(&wren::theme::regular(14.0), '日')),
            "arabic_glyph": ctx.fonts_mut(|f| f.has_glyph(&wren::theme::regular(14.0), 'ع')),
            "pixels_per_point": ctx.pixels_per_point(), "images_viewed": self.loaded,
            "messages": self.app.open.as_ref().and_then(|id| self.app.conversation(id)).map(|c| c.messages.len()),
            "encoded_bytes": bytes, "decoded_bytes": decoded, "image_texture_bytes": textures,
            "all_texture_bytes": texture_total, "texture_count": manager.num_allocated(), "atlas": atlas,
            "median_ui_ms": percentile(0.50), "p95_ui_ms": percentile(0.95), "max_ui_ms": times.last(),
            "immediate_repaint_frames": self.immediate, "repaint_causes": self.causes,
            "system_fonts": self.args.system_fonts, "reduce_texture_memory": self.args.reduce_texture_memory,
        })).unwrap();
        self.output.flush().unwrap();
    }

    fn before(&mut self, ctx: &egui::Context) {
        if matches!(self.args.case, Case::Images) && !self.done_images {
            self.since_image += 1;
            let ready = self.current_uri.as_ref().is_none_or(|uri| {
                matches!(
                    ctx.try_load_texture(uri, Default::default(), Default::default()),
                    Ok(egui::load::TexturePoll::Ready { .. })
                )
            });
            let painted = !self.args.native
                || self.loaded == 0
                || self.paints.load(Ordering::Relaxed) > self.image_paints;
            if self.since_image >= 8 && ready && painted && !ctx.has_pending_images() {
                if self.loaded > 0 {
                    self.snapshot(ctx, "image_loaded");
                }
                self.since_image = 0;
                if self.loaded < self.args.images {
                    let url = format!("https://graph.microsoft.com/fixture/{}.png", self.loaded);
                    self.app.handle(
                        BackendEvent::Media {
                            url: url.clone(),
                            bytes: Some(self.image.clone()),
                        },
                        ctx,
                    );
                    let Image::Ready(uri) = self.app.media[&url].clone() else {
                        panic!("fixture rejected")
                    };
                    self.current_uri = Some(uri.clone());
                    self.image_paints = self.paints.load(Ordering::Relaxed);
                    self.app.actions.push(Action::PreviewImage(uri));
                    self.loaded += 1;
                } else {
                    self.app.actions.push(Action::CloseOverlay);
                    self.app.actions.push(Action::Navigate(Section::Calendar));
                    self.done_images = true;
                }
            }
        }
        if matches!(self.args.case, Case::Views) && self.frames.is_multiple_of(8) {
            let chats: Vec<_> = self.app.chats.iter().map(|c| c.id.clone()).collect();
            let id = chats[(self.frames / 8) % chats.len()].clone();
            self.app
                .actions
                .push(Action::OpenConversation(ConversationId::Chat(id)));
        }
        self.app.observe_window(ctx);
        self.app.background(ctx);
    }

    fn events(&self) -> Vec<Event> {
        match self.args.case {
            Case::Hover if self.frames == 0 => vec![Event::PointerMoved(Pos2::new(800.0, 550.0))],
            Case::Scroll => vec![
                Event::PointerMoved(Pos2::new(900.0, 400.0)),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: Vec2::new(
                        0.0,
                        if self.frames % 180 < 90 {
                            100.0
                        } else {
                            -100.0
                        },
                    ),
                    modifiers: Modifiers::NONE,
                    phase: egui::TouchPhase::Move,
                },
            ],
            _ if self.frames == 0 => vec![Event::PointerGone],
            _ => Vec::new(),
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let started = Instant::now();
        let ctx = ui.ctx().clone();
        self.scrolling.apply(&ctx);
        wren::ui::show(&mut self.app, ui);
        self.app.apply_actions(&ctx);
        if self.args.native {
            let paints = Arc::clone(&self.paints);
            ui.painter().add(egui::PaintCallback {
                rect: ctx.content_rect(),
                callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_, _| {
                    paints.fetch_add(1, Ordering::Relaxed);
                })),
            });
        }
        self.times.push(started.elapsed().as_secs_f64() * 1000.0);
        self.frames += 1;
        if self.frames == 8 {
            self.snapshot(&ctx, "warm");
        }
        for cause in ctx.repaint_causes() {
            *self.causes.entry(format!("{cause:?}")).or_default() += 1;
        }
    }
}

impl eframe::App for Probe {
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        // Keep unrelated typing and pointer movement out of fixture inputs.
        if matches!(self.args.case, Case::Scroll) {
            if self.last_scroll.elapsed() < Duration::from_secs_f64(1.0 / 60.0) {
                raw.events.clear();
                return;
            }
            self.last_scroll = Instant::now();
        }
        raw.events = self.events();
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(pacing) = &mut self.pacing {
            pacing.begin_frame(frame);
        }
        self.before(ctx);
        let remaining =
            Duration::from_secs(self.args.seconds).saturating_sub(self.started.elapsed());
        if remaining.is_zero() {
            self.snapshot(ctx, "end");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.request_repaint_after(remaining);
            if matches!(self.args.case, Case::Scroll | Case::Views)
                || (matches!(self.args.case, Case::Images) && !self.done_images)
            {
                ctx.request_repaint_after(Duration::from_secs_f64(1.0 / 60.0));
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.draw(ui);
    }
}

fn main() -> eframe::Result<()> {
    let args = Args::parse();
    if args.native {
        eframe::run_native(
            "Wren performance fixture",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
                persist_window: false,
                ..Default::default()
            },
            Box::new(move |cc| {
                let mut probe = Probe::new(args, &cc.egui_ctx);
                probe.pacing = Some(wren::pacing::FramePacer::new(cc));
                Ok(Box::new(probe))
            }),
        )
    } else {
        let ctx = egui::Context::default();
        let scale = args.scale;
        let frames = args.frames;
        let mut probe = Probe::new(args, &ctx);
        for index in 0..frames {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(1280.0, 800.0),
                )),
                time: Some(index as f64 / 60.0),
                events: probe.events(),
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(scale);
            let mut output = ctx.run_ui(input, |ui| {
                probe.before(ui.ctx());
                probe.draw(ui);
            });
            if output.viewport_output[&egui::ViewportId::ROOT]
                .repaint_delay
                .is_zero()
            {
                probe.immediate += 1;
            }
            let _ = ctx.tessellate(output.shapes, output.pixels_per_point);
            // Headless measurements have no graphics driver to consume uploads.
            output.textures_delta.clear();
            if ctx.has_pending_images() {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        probe.snapshot(&ctx, "end");
        Ok(())
    }
}
