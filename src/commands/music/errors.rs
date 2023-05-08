use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum MusicCommandError {
    #[error("No estoy tocando nada")]
    NoSongPlaying,
    #[error("Tiempo inválido")]
    InvalidTime,
    #[error("Índice inválido")]
    InvalidQueueIndex,
    #[error("No encontré la canción")]
    FailedVideoSearch,
    #[error("La cola está vacía")]
    EmptyQueue,
    #[error("No estás en un canal de voz")]
    NoVoiceChannel,
    #[error("No me pude unir al canal")]
    FailedToJoinChannel,
    #[error("Este formato no se puede seekear")]
    SeekFailed,
    #[error("Error")]
    Generic,
}
