//! Local image sources, bounded thumbnail textures, and short-lived originals.
//! Decoding runs on one worker, never on the interface thread. No loader fetches
//! a URL: Graph downloads still go through the account backend.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Weak, mpsc};

use egui::load::{
    Bytes, BytesLoadResult, BytesLoader, BytesPoll, ImageLoadResult, ImageLoader, ImagePoll,
    LoadError, SizeHint, SizedTexture, TextureLoadResult, TextureLoader, TexturePoll,
};
use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2, mutex::Mutex};
use image::ImageDecoder as _;

const MIB: usize = 1024 * 1024;
pub const SOURCE_BUDGET: usize = 16 * MIB;
pub const TEXTURE_BUDGET: usize = 16 * MIB;
const ORIGINAL: &str = "#original";
const CROPPED: &str = "#cropped";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Key {
    uri: String,
    /// Zero means original resolution; other values are thumbnail edge buckets.
    edge: u32,
    cropped: bool,
}

impl Key {
    fn new(uri: &str, hint: SizeHint) -> Self {
        let uri = egui::decode_animated_image_uri(uri).map_or(uri, |(uri, _)| uri);
        let edge = if uri.ends_with(ORIGINAL) {
            0
        } else {
            let wanted = match hint {
                SizeHint::Width(w) => w,
                SizeHint::Height(h) => h,
                SizeHint::Size { width, height, .. } => width.max(height),
                SizeHint::Scale(_) => 1024,
            };
            wanted.clamp(64, 1024).next_power_of_two()
        };
        Self {
            uri: uri.to_owned(),
            edge,
            cropped: uri.ends_with(CROPPED),
        }
    }

    fn source(&self) -> &str {
        source_uri(&self.uri)
    }
}

struct Source {
    bytes: Arc<[u8]>,
    used: u64,
}

struct Decoded {
    ticket: u64,
    pass: u64,
    image: Option<Result<Arc<ColorImage>, &'static str>>,
}

struct Texture {
    handle: TextureHandle,
    source_size: Vec2,
    pass: u64,
    used: u64,
}

#[derive(Default)]
struct State {
    sources: HashMap<String, Source>,
    decoded: HashMap<Key, Decoded>,
    textures: HashMap<(Key, TextureOptions), Texture>,
    pass: u64,
    serial: u64,
}

struct Shared {
    state: Mutex<State>,
    jobs: mpsc::SyncSender<Job>,
}

struct Job {
    key: Key,
    ticket: u64,
    bytes: Bytes,
    ctx: Context,
    max_side: u32,
}

fn cache(ctx: &Context) -> Arc<Shared> {
    let id = egui::Id::new("wren-image-cache");
    if let Some(cache) = ctx.data(|d| d.get_temp::<Arc<Shared>>(id)) {
        return cache;
    }
    let (jobs, receiver) = mpsc::sync_channel::<Job>(8);
    let shared = Arc::new(Shared {
        state: Mutex::default(),
        jobs,
    });
    let weak = Arc::downgrade(&shared);
    std::thread::Builder::new()
        .name("wren-images".into())
        .spawn(move || worker(weak, receiver))
        .expect("could not start the image decoder");
    ctx.add_bytes_loader(Arc::new(Sources(Arc::clone(&shared))));
    ctx.add_image_loader(Arc::new(Decoder(Arc::clone(&shared))));
    ctx.add_texture_loader(Arc::new(Textures(Arc::clone(&shared))));
    ctx.data_mut(|d| d.insert_temp(id, Arc::clone(&shared)));
    shared
}

pub fn install(ctx: &Context) {
    let _ = cache(ctx);
}

fn managed(uri: &str) -> bool {
    uri.starts_with("bytes://wren/") || uri.starts_with("bytes://pasted-")
}

fn source_uri(uri: &str) -> &str {
    // egui appends a frame index even when only the static GIF decoder is
    // enabled. Keep the existing first-frame GIF behavior.
    let uri = egui::decode_animated_image_uri(uri).map_or(uri, |(uri, _)| uri);
    uri.strip_suffix(ORIGINAL)
        .or_else(|| uri.strip_suffix(CROPPED))
        .unwrap_or(uri)
}

pub fn original(uri: &str) -> String {
    format!("{}{ORIGINAL}", source_uri(uri))
}

/// A crop needs enough pixels on the short edge, not just the long edge.
pub fn cropped(uri: &str) -> String {
    format!("{}{CROPPED}", source_uri(uri))
}

/// Retain the compressed source so a preview can be decoded again after eviction.
pub fn include(ctx: &Context, uri: String, bytes: impl Into<Arc<[u8]>>) {
    let shared = cache(ctx);
    let mut state = shared.state.lock();
    let bytes = bytes.into();
    let changed = state
        .sources
        .get(&uri)
        .is_none_or(|old| old.bytes.as_ref() != bytes.as_ref());
    if changed {
        state.decoded.retain(|key, _| key.source() != uri);
        state.textures.retain(|(key, _), _| key.source() != uri);
    }
    state.serial += 1;
    let used = state.serial;
    state.sources.insert(uri.clone(), Source { bytes, used });
    let mut total: usize = state.sources.values().map(|s| s.bytes.len()).sum();
    while total > SOURCE_BUDGET && state.sources.len() > 1 {
        let oldest = state
            .sources
            .iter()
            .filter(|(key, _)| **key != uri)
            .min_by_key(|(_, s)| s.used)
            .map(|(key, _)| key.clone());
        let Some(oldest) = oldest else { break };
        if let Some(source) = state.sources.remove(&oldest) {
            total -= source.bytes.len();
        }
    }
}

pub fn contains(ctx: &Context, uri: &str) -> bool {
    cache(ctx)
        .state
        .lock()
        .sources
        .contains_key(source_uri(uri))
}

/// Used when a view discovers that an evicted source needs an ordinary reload.
pub fn needs_source(ctx: &Context, uri: &str, hint: SizeHint) -> bool {
    if contains(ctx, uri) {
        return false;
    }
    let shared = cache(ctx);
    let key = Key::new(uri, hint);
    let state = shared.state.lock();
    !state.textures.keys().any(|(stored, _)| *stored == key)
}

struct Sources(Arc<Shared>);
impl BytesLoader for Sources {
    fn id(&self) -> &str {
        "wren-image-sources"
    }
    fn load(&self, _: &Context, uri: &str) -> BytesLoadResult {
        if !managed(uri) {
            return Err(LoadError::NotSupported);
        }
        let mut state = self.0.state.lock();
        state.serial += 1;
        let used = state.serial;
        let Some(source) = state.sources.get_mut(source_uri(uri)) else {
            return Err(LoadError::NotSupported);
        };
        source.used = used;
        Ok(BytesPoll::Ready {
            size: None,
            bytes: Bytes::Shared(Arc::clone(&source.bytes)),
            mime: None,
        })
    }
    fn forget(&self, uri: &str) {
        self.0.state.lock().sources.remove(source_uri(uri));
    }
    fn forget_all(&self) {
        self.0.state.lock().sources.clear();
    }
    fn byte_size(&self) -> usize {
        self.0
            .state
            .lock()
            .sources
            .values()
            .map(|s| s.bytes.len())
            .sum()
    }
}

struct Decoder(Arc<Shared>);
impl ImageLoader for Decoder {
    fn id(&self) -> &str {
        "wren-image-decoder"
    }
    fn load(&self, ctx: &Context, uri: &str, hint: SizeHint) -> ImageLoadResult {
        if !managed(uri) {
            return Err(LoadError::NotSupported);
        }
        let key = Key::new(uri, hint);
        {
            let mut state = self.0.state.lock();
            let pass = state.pass;
            if let Some(decoded) = state.decoded.get_mut(&key) {
                decoded.pass = pass;
                return match &decoded.image {
                    None => Ok(ImagePoll::Pending { size: None }),
                    Some(Ok(image)) => Ok(ImagePoll::Ready {
                        image: Arc::clone(image),
                    }),
                    Some(Err(error)) => Err(LoadError::Loading((*error).into())),
                };
            }
        }
        let bytes = match ctx.try_load_bytes(key.source())? {
            BytesPoll::Pending { size } => return Ok(ImagePoll::Pending { size }),
            BytesPoll::Ready { bytes, .. } => bytes,
        };
        let ticket = {
            let mut state = self.0.state.lock();
            state.serial += 1;
            let ticket = state.serial;
            let pass = state.pass;
            state.decoded.insert(
                key.clone(),
                Decoded {
                    ticket,
                    pass,
                    image: None,
                },
            );
            ticket
        };
        if self
            .0
            .jobs
            .try_send(Job {
                key: key.clone(),
                ticket,
                bytes,
                ctx: ctx.clone(),
                max_side: ctx.input(|i| i.max_texture_side.min(u32::MAX as usize) as u32),
            })
            .is_err()
        {
            self.0.state.lock().decoded.remove(&key);
        }
        Ok(ImagePoll::Pending { size: None })
    }
    fn forget(&self, uri: &str) {
        self.0
            .state
            .lock()
            .decoded
            .retain(|key, _| source_uri(&key.uri) != source_uri(uri));
    }
    fn forget_all(&self) {
        self.0.state.lock().decoded.clear();
    }
    fn byte_size(&self) -> usize {
        self.0
            .state
            .lock()
            .decoded
            .values()
            .map(|entry| match &entry.image {
                Some(Ok(image)) => image.pixels.len() * 4,
                _ => 0,
            })
            .sum()
    }
    fn has_pending(&self) -> bool {
        self.0
            .state
            .lock()
            .decoded
            .values()
            .any(|d| d.image.is_none())
    }
}

fn worker(shared: Weak<Shared>, receiver: mpsc::Receiver<Job>) {
    while let Ok(job) = receiver.recv() {
        let pending = shared.upgrade().is_some_and(|s| {
            s.state
                .lock()
                .decoded
                .get(&job.key)
                .is_some_and(|d| d.ticket == job.ticket)
        });
        if !pending {
            continue;
        }
        let image = decode(&job.bytes, job.key.edge, job.key.cropped, job.max_side).map(Arc::new);
        let changed = shared.upgrade().is_some_and(|shared| {
            let mut state = shared.state.lock();
            if let Some(entry) = state
                .decoded
                .get_mut(&job.key)
                .filter(|d| d.ticket == job.ticket)
            {
                entry.image = Some(image);
                true
            } else {
                false
            }
        });
        if changed {
            job.ctx.request_repaint();
        }
    }
}

fn decode(
    bytes: &[u8],
    edge: u32,
    cropped: bool,
    max_side: u32,
) -> Result<ColorImage, &'static str> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "Image could not be decoded.")?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(128 * MIB as u64);
    reader.limits(limits);
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| "Image could not be decoded.")?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut decoded =
        image::DynamicImage::from_decoder(decoder).map_err(|_| "Image could not be decoded.")?;
    decoded.apply_orientation(orientation);
    let source_size = Vec2::new(decoded.width() as f32, decoded.height() as f32);
    let edge = if edge == 0 {
        max_side
    } else {
        let edge = if cropped {
            let long = decoded.width().max(decoded.height()) as f64;
            let short = decoded.width().min(decoded.height()).max(1) as f64;
            (f64::from(edge) * long / short).ceil().min(4096.0) as u32
        } else {
            edge
        };
        edge.min(max_side)
    }
    .max(1);
    let rgba = if decoded.width().max(decoded.height()) > edge {
        decoded.thumbnail(edge, edge)
    } else {
        decoded
    }
    .into_rgba8();
    let mut image =
        ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], &rgba);
    image.source_size = source_size;
    Ok(image)
}

struct Textures(Arc<Shared>);
impl TextureLoader for Textures {
    fn id(&self) -> &str {
        "wren-image-textures"
    }
    fn load(
        &self,
        ctx: &Context,
        uri: &str,
        options: TextureOptions,
        hint: SizeHint,
    ) -> TextureLoadResult {
        if !managed(uri) {
            return Err(LoadError::NotSupported);
        }
        let key = Key::new(uri, hint);
        {
            let mut state = self.0.state.lock();
            state.serial += 1;
            let used = state.serial;
            let pass = state.pass;
            if let Some(source) = state.sources.get_mut(key.source()) {
                source.used = used;
            }
            if let Some(texture) = state.textures.get_mut(&(key.clone(), options)) {
                texture.used = used;
                texture.pass = pass;
                return Ok(TexturePoll::Ready {
                    texture: SizedTexture::new(texture.handle.id(), texture.source_size),
                });
            }
        }
        match ctx.try_load_image(uri, hint)? {
            ImagePoll::Pending { size } => Ok(TexturePoll::Pending { size }),
            ImagePoll::Ready { image } => {
                let source_size = image.source_size;
                let handle = ctx.load_texture(source_uri(uri), image, options);
                let texture = SizedTexture::new(handle.id(), source_size);
                let mut state = self.0.state.lock();
                state.decoded.remove(&key);
                let pass = state.pass;
                let used = state.serial;
                state.textures.insert(
                    (key, options),
                    Texture {
                        handle,
                        source_size,
                        pass,
                        used,
                    },
                );
                Ok(TexturePoll::Ready { texture })
            }
        }
    }
    fn forget(&self, uri: &str) {
        self.0
            .state
            .lock()
            .textures
            .retain(|(key, _), _| source_uri(&key.uri) != source_uri(uri));
    }
    fn forget_all(&self) {
        self.0.state.lock().textures.clear();
    }
    fn end_pass(&self, _: u64) {
        let mut state = self.0.state.lock();
        let pass = state.pass;
        state
            .textures
            .retain(|(key, _), entry| key.edge != 0 || entry.pass == pass);
        let mut total: usize = state.textures.values().map(|t| t.handle.byte_size()).sum();
        while total > TEXTURE_BUDGET {
            let oldest = state
                .textures
                .iter()
                .filter(|(_, entry)| entry.pass < pass)
                .min_by_key(|(_, entry)| entry.used)
                .map(|(key, _)| key.clone());
            let Some(oldest) = oldest else { break };
            if let Some(texture) = state.textures.remove(&oldest) {
                total -= texture.handle.byte_size();
            }
        }
        state
            .decoded
            .retain(|_, entry| entry.image.is_none() || entry.pass >= pass);
        state.pass += 1;
    }
    fn byte_size(&self) -> usize {
        self.0
            .state
            .lock()
            .textures
            .values()
            .map(|t| t.handle.byte_size())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(1920, 1080, image::Rgba([30, 50, 70, 255]))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn thumbnails_keep_original_dimensions_and_originals_remain_available() {
        let bytes = png();
        let thumbnail = decode(&bytes, 256, false, 16384).unwrap();
        assert_eq!(thumbnail.size, [256, 144]);
        assert_eq!(thumbnail.source_size, Vec2::new(1920.0, 1080.0));
        assert_eq!(decode(&bytes, 0, false, 16384).unwrap().size, [1920, 1080]);
        assert!(decode(b"not an image", 256, false, 16384).is_err());
    }

    #[test]
    fn jpeg_orientation_applies_to_thumbnails_and_originals() {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(80, 40, image::Rgb([30, 50, 70]))
            .write_to(&mut bytes, image::ImageFormat::Jpeg)
            .unwrap();
        let mut bytes = bytes.into_inner();
        // APP1 EXIF, little-endian TIFF, orientation 6 (90 degrees clockwise).
        let exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        segment.extend_from_slice(exif);
        bytes.splice(2..2, segment);
        assert_eq!(decode(&bytes, 0, false, 16384).unwrap().size, [40, 80]);
        let thumb = decode(&bytes, 20, false, 16384).unwrap();
        assert_eq!(thumb.size, [10, 20]);
        assert_eq!(thumb.source_size, Vec2::new(40.0, 80.0));
    }

    #[test]
    fn cropped_panorama_keeps_detail_on_the_short_edge() {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(3072, 256, image::Rgba([30, 50, 70, 255]))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let bytes = bytes.into_inner();
        assert_eq!(decode(&bytes, 256, false, 16384).unwrap().size, [256, 21]);
        let crop = decode(&bytes, 256, true, 16384).unwrap();
        assert_eq!(crop.size, [3072, 256]);
        assert_eq!(crop.source_size, Vec2::new(3072.0, 256.0));
        assert_eq!(decode(&bytes, 256, true, 2048).unwrap().size, [2048, 171]);
        assert_eq!(
            source_uri(&cropped("bytes://pasted-test.png")),
            "bytes://pasted-test.png"
        );
    }

    #[test]
    fn gif_frame_suffixes_use_the_same_source_and_texture() {
        let hint = SizeHint::Width(256);
        assert_eq!(
            Key::new("bytes://wren/test.gif", hint),
            Key::new("bytes://wren/test.gif#0", hint)
        );
        assert_eq!(
            source_uri("bytes://wren/test.gif#original#0"),
            "bytes://wren/test.gif"
        );
    }

    fn ready(ctx: &Context, uri: &str) -> SizedTexture {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match ctx
                .try_load_texture(uri, TextureOptions::default(), SizeHint::Width(256))
                .unwrap()
            {
                TexturePoll::Ready { texture } => return texture,
                TexturePoll::Pending { .. } => {
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
        }
    }

    #[test]
    fn originals_are_released_after_leaving_and_can_be_reopened_without_network() {
        let ctx = Context::default();
        let uri = "bytes://wren/test.png";
        include(&ctx, uri.into(), png());
        let thumbnail = ready(&ctx, uri);
        let full = ready(&ctx, &original(uri));
        assert_ne!(thumbnail.id, full.id);
        let shared = cache(&ctx);
        assert_eq!(Decoder(Arc::clone(&shared)).byte_size(), 0);
        let textures = Textures(Arc::clone(&shared));
        textures.end_pass(0);
        ready(&ctx, uri);
        textures.end_pass(1);
        assert!(
            shared
                .state
                .lock()
                .textures
                .keys()
                .all(|(key, _)| key.edge != 0)
        );
        assert_eq!(ready(&ctx, &original(uri)).size, Vec2::new(1920.0, 1080.0));
        assert!(contains(&ctx, uri));
        ctx.tex_manager().write().take_delta().clear();
    }

    #[test]
    fn compressed_sources_and_offscreen_textures_have_budgets() {
        let ctx = Context::default();
        for i in 0..20 {
            include(&ctx, format!("bytes://wren/{i}.png"), vec![0; MIB]);
        }
        let shared = cache(&ctx);
        assert!(Sources(Arc::clone(&shared)).byte_size() <= SOURCE_BUDGET);
        assert!(!contains(&ctx, "bytes://wren/0.png"));
        assert!(contains(&ctx, "bytes://wren/19.png"));
        {
            let mut state = shared.state.lock();
            state.pass = 1;
            for i in 0..10 {
                let handle = ctx.load_texture(
                    "fixture",
                    ColorImage::filled([1024, 1024], egui::Color32::BLACK),
                    TextureOptions::default(),
                );
                state.textures.insert(
                    (
                        Key::new(&format!("bytes://wren/{i}.png"), SizeHint::Width(1024)),
                        TextureOptions::default(),
                    ),
                    Texture {
                        handle,
                        source_size: Vec2::splat(1024.0),
                        pass: 0,
                        used: i,
                    },
                );
            }
        }
        let textures = Textures(shared);
        textures.end_pass(1);
        assert!(textures.byte_size() <= TEXTURE_BUDGET);
        ctx.tex_manager().write().take_delta().clear();
    }
}
