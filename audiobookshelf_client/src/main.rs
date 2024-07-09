use anyhow::Result;
use audiobookshelf_api::params::{DeviceInfoParams, PlayLibraryItemParams};
use audiobookshelf_api::schema::PlaybackSessionExtended;
use audiobookshelf_api::stream_download::storage::temp::TempStorageProvider;
use audiobookshelf_api::stream_download::StreamDownload;
use audiobookshelf_api::{
    schema::{AudioTrack, FileMetadata},
    ClientConfig, Url, UserClient,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rodio::{source::EmptyCallback, Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fs::File;
use std::future::IntoFuture;
use std::io::{BufReader, Read, Seek};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

macro_rules! unwrap_or_return {
    ($option:expr, $result:expr) => {
        if let Some(value) = $option {
            value
        } else {
            return $result;
        }
    };
}

struct ApiError(anyhow::Error);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    let config = ClientConfig {
        root_url: Url::parse(&var("AUDIOBOOKSHELF_URL")?)?,
    };
    let username = var("AUDIOBOOKSHELF_USERNAME")?;
    let password = var("AUDIOBOOKSHELF_PASSWORD")?;
    let listen_on = var("AUDIOBOOKSHELF_CLIENT_LISTEN")?;
    let client = UserClient::auth(config, username, password).await?;

    // Initialize audio player
    let mut client = AudioClient::new(client)?;
    client.use_local(true);
    client.set_current_item().await?;
    client.sink.play();

    // Connect player to server
    let (send, recv) = mpsc::channel(512);

    // Launch control server
    let listener = tokio::net::TcpListener::bind(&listen_on).await.unwrap();
    let app = Router::new()
        .route("/play/", post(play))
        .route("/position/", post(seek))
        .route("/position/", get(get_position))
        .route("/volume/", post(set_volume))
        .route("/volume/", get(get_volume))
        .with_state(send);

    tokio::select! {
        result = run_audio_client(&mut client, recv) => {
            result?;
        },
        result = axum::serve(listener, app).into_future() => {
            result?;
        }
    };

    Ok(())
}

#[derive(Deserialize)]
struct SetPlayRequest {
    play: bool,
}

async fn play(
    State(sender): State<mpsc::Sender<ClientEvent>>,
    Json(data): Json<SetPlayRequest>,
) -> StatusCode {
    let event = if data.play {
        ClientEvent::Play
    } else {
        ClientEvent::Pause
    };
    match sender.send(event).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

#[derive(Deserialize)]
struct SeekRequest {
    offset: f64,
}

async fn seek(
    State(sender): State<mpsc::Sender<ClientEvent>>,
    Json(data): Json<SeekRequest>,
) -> StatusCode {
    match sender.send(ClientEvent::Seek(data.offset)).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

async fn get_position(
    State(sender): State<mpsc::Sender<ClientEvent>>,
) -> Result<Json<PositionOffset>, ApiError> {
    let (return_sender, receiver) = oneshot::channel();
    sender.send(ClientEvent::GetOffset(return_sender)).await?;

    let result = receiver
        .await?
        .ok_or_else(|| anyhow::anyhow!("Channel is closed"))?;
    Ok(Json(result))
}

#[derive(Deserialize, Serialize)]
struct Volume {
    volume: f32,
}

async fn set_volume(
    State(sender): State<mpsc::Sender<ClientEvent>>,
    Json(data): Json<Volume>,
) -> StatusCode {
    match sender.send(ClientEvent::Volume(data.volume)).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

async fn get_volume(
    State(sender): State<mpsc::Sender<ClientEvent>>,
) -> Result<Json<Volume>, ApiError> {
    let (return_sender, receiver) = oneshot::channel();
    sender.send(ClientEvent::GetVolume(return_sender)).await?;
    let volume = receiver.await?;
    Ok(Json(Volume { volume }))
}

struct AudioClient {
    client: UserClient,
    playing: Option<PlayingState>,
    use_local: bool,
    sink: Arc<Sink>,
    /// Must be present even if not used.
    /// Dropping this value breaks `sink`
    _stream: OutputStream,
}

struct PlayingState {
    playback: PlaybackSessionExtended,
    current_track: usize,
}

#[derive(Serialize)]
struct PositionOffset {
    offset: f64,
    duration: f64,
}

enum ClientEvent {
    Play,
    Pause,
    Seek(f64),
    Volume(f32),
    GetVolume(oneshot::Sender<f32>),
    GetOffset(oneshot::Sender<Option<PositionOffset>>),
}

async fn run_audio_client(
    client: &mut AudioClient,
    mut events: mpsc::Receiver<ClientEvent>,
) -> Result<()> {
    let mut on_audio_end = client.wait_till_end();
    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    Some(ClientEvent::Play) => { client.sink.play(); },
                    Some(ClientEvent::Pause) => { client.sink.pause(); },
                    Some(ClientEvent::Seek(offset)) => {
                        client.seek(offset).await?;
                        on_audio_end = client.wait_till_end();
                    },
                    Some(ClientEvent::Volume(volume)) => {
                        client.sink.set_volume(volume)
                    },
                    Some(ClientEvent::GetVolume(sender)) => {
                        let _ = sender.send(client.get_volume());
                    }
                    Some(ClientEvent::GetOffset(sender)) => {
                        let _ = sender.send(client.get_offset());
                    }
                    None => { return Ok(()); }
                }
            },
            is_finished = on_audio_end.recv() => {
                if is_finished.is_some() {
                    client.sink.clear();
                    client.add_next_track().await?;
                    on_audio_end = client.wait_till_end();
                }
            }
        }
    }
}

impl AudioClient {
    fn new(client: UserClient) -> Result<Self> {
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let sink = Arc::new(rodio::Sink::try_new(&handle)?);
        Ok(Self {
            client,
            sink,
            playing: None,
            use_local: false,
            _stream,
        })
    }

    /// Then set to `true`, player will assume that it executed on same machine as `audiobookshelf` server,
    /// and will try to load audio files directly from file system, instead of proxying through server.
    fn use_local(&mut self, use_local: bool) {
        self.use_local = use_local;
    }

    /// Pause execution until audio file fully played.
    ///
    /// Will immediatly file if sink is cleaned
    fn wait_till_end(&self) -> mpsc::Receiver<()> {
        let (sender, receiver) = mpsc::channel(1);
        self.sink
            .append(EmptyCallback::<f32>::new(Box::new(move || {
                let _ = sender.try_send(());
            })));

        receiver
    }

    fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    fn get_offset(&self) -> Option<PositionOffset> {
        self.playing.as_ref().map(|p| PositionOffset {
            offset: p.playback.audio_tracks[p.current_track].start_offset
                + self.sink.get_pos().as_secs_f64(),
            duration: p.playback.playback_session.duration,
        })
    }

    fn playback_params() -> PlayLibraryItemParams {
        PlayLibraryItemParams {
            device_info: DeviceInfoParams {
                client_name: Some("hukumkas_client".into()),
                ..Default::default()
            },
            supported_mime_types: vec![
                "audio/flac".into(),
                "audio/mpeg".into(),
                "audio/ogg".into(),
            ],
            ..Default::default()
        }
    }

    /// Seek to position.
    /// Position is measured in seconds from beginning of audiobook.
    async fn seek(&mut self, position: f64) -> Result<bool> {
        let playing = if let Some(playing) = &self.playing {
            playing
        } else {
            return Ok(false);
        };
        let (current_track, offset) =
            Self::get_active_track_index(&playing.playback, position).unwrap();
        if current_track != playing.current_track {
            let is_paused = self.sink.is_paused();
            self.sink.clear();
            self.sink.append(Decoder::new(
                self.get_audio_source(&playing.playback.audio_tracks[current_track])
                    .await?,
            )?);
            if !is_paused {
                self.sink.play();
            }
        }
        self.sink
            .try_seek(Duration::from_secs_f64(offset))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(true)
    }

    async fn add_next_track(&mut self) -> Result<bool> {
        let playing = unwrap_or_return!(&mut self.playing, Ok(false));
        if playing.current_track >= playing.playback.audio_tracks.len() {
            return Ok(false);
        }
        playing.current_track += 1;

        let playing = unwrap_or_return!(&self.playing, Ok(false));
        self.sink.append(Decoder::new(
            self.get_audio_source(&playing.playback.audio_tracks[playing.current_track])
                .await?,
        )?);

        Ok(true)
    }

    /// Init sink with current item
    async fn set_current_item(&mut self) -> Result<bool> {
        let current_library_item =
            unwrap_or_return!(self.client.me().await?.currently_listening(), Ok(false));

        let playback = self
            .client
            .library_item_play(&current_library_item, &Self::playback_params())
            .await?;

        let (current_track, offset) =
            Self::get_active_track_index(&playback, playback.playback_session.current_time)
                .unwrap();
        self.sink.clear();
        self.sink.append(Decoder::new(
            self.get_audio_source(&playback.audio_tracks[current_track])
                .await?,
        )?);
        self.sink
            .try_seek(Duration::from_secs_f64(offset))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        self.playing = Some(PlayingState {
            playback,
            current_track,
        });
        Ok(true)
    }

    async fn get_audio_source(&self, track: &AudioTrack) -> Result<Box<dyn ReadSeekMarker>> {
        let source = if self.use_local {
            open_local_stream(&track.metadata)
        } else {
            None
        };
        let result = if let Some(source) = source {
            source
        } else {
            Box::new(self.client.audiofile_stream(&track.content_url).await?)
        };
        Ok(result)
    }

    fn get_active_track_index(
        playback: &PlaybackSessionExtended,
        current_time: f64,
    ) -> Option<(usize, f64)> {
        for (index, track) in playback.audio_tracks.iter().enumerate() {
            if track.start_offset + track.duration >= current_time {
                return Some((index, current_time - track.start_offset));
            }
        }
        None
    }
}

fn open_local_stream(metadata: &Option<FileMetadata>) -> Option<Box<dyn ReadSeekMarker>> {
    let metadata = metadata.as_ref()?;
    let file = BufReader::new(File::open(&metadata.path).ok()?);
    let file_box: Box<dyn ReadSeekMarker> = Box::new(file);
    Some(file_box)
}

trait ReadSeekMarker: Read + Seek + Send + Sync {}

impl<T: Read + Seek + Send + Sync> ReadSeekMarker for BufReader<T> {}
impl ReadSeekMarker for StreamDownload<TempStorageProvider> {}
