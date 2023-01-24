use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone, Copy)]
pub enum MusicCommandError {
    NoSongPlaying,
    InvalidTime,
    InvalidQueueIndex,
    FailedVideoSearch,
    EmptyQueue,
    NoVoiceChannel,
    FailedToJoinChannel,
    NotInVoiceChannel,
    SeekFailed,
}

impl From<MusicCommandError> for &'static str {
    fn from(error: MusicCommandError) -> Self {
        match error {
            MusicCommandError::NoSongPlaying => "No estoy tocando nada",
            MusicCommandError::InvalidTime => "Tiempo Inválido",
            MusicCommandError::InvalidQueueIndex => "Índice inválido",
            MusicCommandError::FailedVideoSearch => "No encontré la canción",
            MusicCommandError::EmptyQueue => "La cola está vacía",
            MusicCommandError::NoVoiceChannel => "No estás en un canal de voz",
            MusicCommandError::FailedToJoinChannel => "No me pude unir al canal",
            MusicCommandError::NotInVoiceChannel => "No estoy en un canal de voz",
            MusicCommandError::SeekFailed => "Este formato no se puede seekear",
        }
    }
}

impl Display for MusicCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl Error for MusicCommandError {}
