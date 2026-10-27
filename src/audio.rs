use std::{
    fs::File,
    io::Read,
    num::NonZero,
    path::Path,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::Sender,
    },
    time::Duration,
};

use rodio::Source;
use symphonia::core::codecs::{CODEC_TYPE_NULL, CodecRegistry, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::{audio::SampleBuffer, units::Time};
use symphonia_adapter_libopus::OpusDecoder;

use crate::stream::{ProgressiveReader, StreamState};

static CODEC_REGISTRY: LazyLock<CodecRegistry> = LazyLock::new(|| {
    let mut registry = CodecRegistry::new();
    symphonia::default::register_enabled_codecs(&mut registry);
    registry.register_all::<OpusDecoder>();
    registry
});

pub enum AudioCommand {
    Play(std::path::PathBuf),
    PlayStream { url: String, hint_extension: String },
    ResumePause,
    Resume,
    Pause,
    SetVolume(f32),
    Seek(f32),
    Stop,
}

pub struct SymphoniaSource {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_buf: Option<SampleBuffer<f32>>,
    sample_index: usize,
    sample_rate: NonZero<u32>,
    channels: NonZero<u16>,
    total_duration: Option<Duration>,
    progress_ms: Arc<AtomicU64>,
    samples_played: u64,
}

impl SymphoniaSource {
    pub fn new<P: AsRef<Path>>(
        path: P,
        progress_ms: Arc<AtomicU64>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file = File::open(path.as_ref())?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.as_ref().extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        Self::init(mss, hint, progress_ms)
    }

    pub fn new_streaming(
        reader: ProgressiveReader,
        hint_extension: &str,
        progress_ms: Arc<AtomicU64>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mss = MediaSourceStream::new(Box::new(reader), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(hint_extension);

        Self::init(mss, hint, progress_ms)
    }

    fn init(
        mss: MediaSourceStream,
        hint: Hint,
        progress_ms: Arc<AtomicU64>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts = MetadataOptions::default();
        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;
        let reader = probed.format;
        let track = reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| SymphoniaError::Unsupported("No supported audio track found"))?;
        let track_id = track.id;
        let codec_params = track.codec_params.clone();
        let sample_rate = NonZero::new(codec_params.sample_rate.unwrap_or(44100))
            .unwrap_or(NonZero::new(44100).unwrap());
        let channels = NonZero::new(codec_params.channels.map(|c| c.count() as u16).unwrap_or(2))
            .unwrap_or(NonZero::new(2).unwrap());
        let total_duration =
            if let (Some(n_frames), Some(tb)) = (codec_params.n_frames, codec_params.time_base) {
                let time = tb.calc_time(n_frames);
                Some(Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac))
            } else {
                None
            };
        let decoder_opts = DecoderOptions::default();
        let decoder = CODEC_REGISTRY.make(&codec_params, &decoder_opts)?;
        Ok(Self {
            reader,
            decoder,
            track_id,
            sample_buf: None,
            sample_index: 0,
            sample_rate,
            channels,
            total_duration,
            progress_ms,
            samples_played: 0,
        })
    }

    fn fill_next_packet(&mut self) -> Result<bool, SymphoniaError> {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(false);
                }
                Err(err) => return Err(err),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut sample_buf = self.sample_buf.take().unwrap_or_else(|| {
                        SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec())
                    });
                    sample_buf.copy_interleaved_ref(decoded);
                    self.sample_buf = Some(sample_buf);
                    self.sample_index = 0;
                    return Ok(true);
                }
                Err(SymphoniaError::DecodeError(err)) => {
                    tracing::warn!(?err, "Skipping corrupted audio packet");
                    continue;
                }
                Err(err) => return Err(err),
            }
        }
    }

    pub fn seek(&mut self, time_secs: f32) -> Result<(), SymphoniaError> {
        let time = Time::from(time_secs as f64);
        self.reader.seek(
            symphonia::core::formats::SeekMode::Accurate,
            symphonia::core::formats::SeekTo::Time {
                time,
                track_id: Some(self.track_id),
            },
        )?;

        self.decoder.reset();
        self.sample_buf = None;
        self.sample_index = 0;

        let total_samples_per_sec = (self.sample_rate.get() as u64) * (self.channels.get() as u64);
        self.samples_played = (time_secs as f64 * total_samples_per_sec as f64) as u64;
        self.progress_ms.store(
            (time_secs * 1000.0) as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        Ok(())
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref buf) = self.sample_buf {
                let samples = buf.samples();
                if self.sample_index < samples.len() {
                    let sample = samples[self.sample_index];
                    self.sample_index += 1;

                    self.samples_played += 1;
                    // Sync for UI, only every 1000 samples
                    if self.samples_played % 1000 == 0 {
                        let total_samples_per_sec =
                            (self.sample_rate.get() as u64) * (self.channels.get() as u64);
                        if total_samples_per_sec > 0 {
                            let ms = (self.samples_played * 1000) / total_samples_per_sec;
                            self.progress_ms
                                .store(ms, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    return Some(sample);
                }
            }

            match self.fill_next_packet() {
                Ok(true) => {}
                Ok(false) => return None,
                Err(err) => {
                    tracing::error!(?err, "Failed to decode packet");
                    return None;
                }
            }
        }
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        if let Some(ref buf) = self.sample_buf {
            let remaining = buf.samples().len().saturating_sub(self.sample_index);
            Some(remaining)
        } else {
            None
        }
    }

    fn channels(&self) -> NonZero<u16> {
        self.channels
    }

    fn sample_rate(&self) -> NonZero<u32> {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}

pub fn init_audio(
    ui_sender: async_channel::Sender<crate::model::UiMediaEvent>,
) -> (Sender<AudioCommand>, Arc<AtomicU64>) {
    let (sender, receiver) = std::sync::mpsc::channel::<AudioCommand>();
    let progress_ms = Arc::new(AtomicU64::new(0));
    let progress_for_thread = progress_ms.clone();

    std::thread::spawn(move || {
        let mut sink_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        sink_handle.log_on_drop(false);

        let mut current_player: Option<rodio::Player> = None;
        let mut current_path: Option<std::path::PathBuf> = None;
        let mut current_volume = 1.0f32;
        let mut has_played = false;

        let mut current_stream_state: Option<Arc<StreamState>> = None;
        let mut _current_temp_file: Option<tempfile::NamedTempFile> = None;

        loop {
            let command = if current_player.is_some() {
                match receiver.recv_timeout(Duration::from_millis(50)) {
                    Ok(cmd) => Some(cmd),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if has_played {
                            if let Some(ref player) = current_player {
                                if !player.is_paused() && player.empty() {
                                    has_played = false;
                                    current_player = None;
                                    current_path = None;
                                    progress_for_thread
                                        .store(0, std::sync::atomic::Ordering::Relaxed);
                                    let _ = ui_sender
                                        .send_blocking(crate::model::UiMediaEvent::TrackEnded);
                                }
                            }
                        }
                        None
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                match receiver.recv() {
                    Ok(cmd) => Some(cmd),
                    Err(_) => break,
                }
            };

            let Some(command) = command else {
                continue;
            };

            match command {
                AudioCommand::Play(path) => {
                    if let Some(state) = current_stream_state.take() {
                        state.is_cancelled.store(true, Ordering::Relaxed);
                    }
                    _current_temp_file = None;

                    progress_for_thread.store(0, std::sync::atomic::Ordering::Relaxed);
                    current_path = Some(path.clone());

                    match SymphoniaSource::new(&path, progress_for_thread.clone()) {
                        Ok(source) => {
                            let player = rodio::Player::connect_new(&sink_handle.mixer());
                            player.set_volume(current_volume);
                            player.append(source);
                            current_player = Some(player);
                            has_played = true;
                        }
                        Err(err) => {
                            tracing::error!(?err, "Failed to load audio file with symphonia");
                            has_played = false;
                            current_player = None;
                        }
                    }
                }
                AudioCommand::PlayStream {
                    url,
                    hint_extension,
                } => {
                    if let Some(state) = current_stream_state.take() {
                        state.is_cancelled.store(true, Ordering::Relaxed);
                    }
                    _current_temp_file = None;

                    let Ok(temp_file) = tempfile::Builder::new()
                        .suffix(&format!(".{}", hint_extension))
                        .tempfile()
                    else {
                        tracing::error!("Failed to create temporary file for streaming");
                        continue;
                    };
                    let temp_path = temp_file.path().to_path_buf();

                    let state = Arc::new(StreamState {
                        downloaded_bytes: AtomicUsize::new(0),
                        is_finished: AtomicBool::new(false),
                        is_cancelled: AtomicBool::new(false),
                    });

                    let downloaded_state = state.clone();
                    let downloaded_path = temp_path.clone();

                    std::thread::spawn(move || {
                        use std::io::Write;

                        let mut writer = std::fs::OpenOptions::new()
                            .write(true)
                            .open(&downloaded_path)
                            .unwrap();

                        let mut response = match reqwest::blocking::get(&url) {
                            Ok(res) => res,
                            Err(err) => {
                                tracing::error!(?err, "Download failed");
                                downloaded_state.is_finished.store(true, Ordering::Relaxed);
                                return;
                            }
                        };

                        let mut buffer = [0u8; 8192];
                        loop {
                            if downloaded_state.is_cancelled.load(Ordering::Relaxed) {
                                break;
                            }

                            match response.read(&mut buffer) {
                                // Download Finished
                                Ok(0) => break,
                                Ok(n) => {
                                    writer.write_all(&buffer[..n]).unwrap();
                                    writer.flush().unwrap();
                                    downloaded_state
                                        .downloaded_bytes
                                        .fetch_add(n, Ordering::Relaxed);
                                }
                                Err(_) => break,
                            }
                        }
                        downloaded_state.is_finished.store(true, Ordering::Relaxed);
                    });

                    current_stream_state = Some(state.clone());
                    current_path = Some(temp_path.clone());
                    _current_temp_file = Some(temp_file);

                    progress_for_thread.store(0, Ordering::Relaxed);

                    // Wait 256kb or download is finished
                    while state.downloaded_bytes.load(Ordering::Relaxed) < 256 * 1024
                        && !state.is_finished.load(Ordering::Relaxed)
                    {
                        if state.is_cancelled.load(Ordering::Relaxed) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }

                    if state.is_cancelled.load(Ordering::Relaxed) {
                        continue;
                    }

                    let Ok(read_file) = std::fs::File::open(&temp_path) else {
                        tracing::error!("Could not open temp stream file for reading");
                        continue;
                    };
                    let progressive_reader =
                        crate::stream::ProgressiveReader::new(read_file, state.clone());
                    match SymphoniaSource::new_streaming(
                        progressive_reader,
                        &hint_extension,
                        progress_for_thread.clone(),
                    ) {
                        Ok(source) => {
                            let player = rodio::Player::connect_new(&sink_handle.mixer());
                            player.set_volume(current_volume);
                            player.append(source);
                            current_player = Some(player);
                            has_played = true;
                        }
                        Err(err) => {
                            tracing::error!(?err, "Failed to load audio stream with symphonia");
                            has_played = false;
                            current_player = None;
                        }
                    }
                }
                AudioCommand::ResumePause => {
                    if let Some(ref player) = current_player {
                        if player.is_paused() {
                            player.play();
                        } else {
                            player.pause();
                        }
                    }
                }
                AudioCommand::Resume => {
                    if let Some(ref player) = current_player {
                        player.play();
                    }
                }
                AudioCommand::Pause => {
                    if let Some(ref player) = current_player {
                        player.pause();
                    }
                }
                AudioCommand::SetVolume(vol) => {
                    current_volume = vol;
                    if let Some(ref player) = current_player {
                        player.set_volume(vol);
                    }
                }
                AudioCommand::Seek(time_secs) => {
                    if let Some(ref path) = current_path {
                        let source_res = if let Some(ref state) = current_stream_state {
                            match std::fs::File::open(path) {
                                Ok(file) => {
                                    let reader =
                                        crate::stream::ProgressiveReader::new(file, state.clone());
                                    let ext =
                                        path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                    SymphoniaSource::new_streaming(
                                        reader,
                                        ext,
                                        progress_for_thread.clone(),
                                    )
                                }
                                Err(err) => {
                                    Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
                                }
                            }
                        } else {
                            SymphoniaSource::new(path, progress_for_thread.clone())
                        };

                        if let Ok(mut source) = source_res {
                            if let Ok(()) = source.seek(time_secs) {
                                if let Some(player) = current_player.take() {
                                    player.stop();
                                }
                                let player = rodio::Player::connect_new(&sink_handle.mixer());
                                player.set_volume(current_volume);
                                player.append(source);
                                current_player = Some(player);
                                has_played = true;
                            }
                        }
                    }
                }
                AudioCommand::Stop => {
                    if let Some(state) = current_stream_state.take() {
                        state.is_cancelled.store(true, Ordering::Relaxed);
                    }
                    _current_temp_file = None;

                    if let Some(ref player) = current_player {
                        player.stop();
                    }
                    has_played = false;
                    current_player = None;
                    current_path = None;
                    progress_for_thread.store(0, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    });

    (sender, progress_ms)
}
