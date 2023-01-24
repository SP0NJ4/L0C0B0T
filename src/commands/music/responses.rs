// Responses for the music commands

use std::time::Duration;

use serenity::{
    builder::CreateEmbed,
    model::user::User,
    prelude::{Context, Mentionable},
    utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::tracks::TrackHandle;

use crate::globals::PRIMARY_COLOR;

use super::queue::{TrackChannel, TrackRequester};

/// Converts a duration to a string in the format `mm:ss`
///
/// ## Arguments
///
/// * `duration` - The duration to convert
///
/// ## Returns
///
/// * `String` - The duration in the format `hh:mm:ss`
fn duration_to_minutes(duration: &Duration) -> String {
    let seconds = duration.as_secs();

    if seconds > 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}

/// Returns the custom track metadata used in response messages. This includes the requester and the
/// channel the song was requested in.
///
/// ## Arguments
///
/// * `ctx` - The context of the command
/// * `track` - The track to get the metadata from
///
/// ## Returns
///
/// * `(User, String)` - The requester and the channel name
async fn get_custom_metadata(ctx: &Context, track: &TrackHandle) -> (User, String) {
    let (requester_id, channel_id) = {
        let typemap = track.typemap().read().await;

        let requester_id = typemap.get::<TrackRequester>().unwrap().clone();
        let channel_id = typemap.get::<TrackChannel>().unwrap().clone();

        (requester_id, channel_id)
    };

    let requester = requester_id.to_user(ctx).await.unwrap();
    let channel_name = channel_id.name(ctx).await.unwrap();

    (requester, channel_name)
}

/// Returns a string with a bar that represents the current position of the track.
///
/// ## Arguments
///
/// * `length` - The length of the bar
/// * `ratio` - The position of the track, from 0 to 1
///
/// ## Returns
///
/// * `String` - The bar
fn playing_bar(length: usize, ratio: f32) -> String {
    let mut bar = String::new();

    let before = (length as f32 * ratio - 1.0).round() as usize;
    let after = length - before - 1;

    bar.push_str("▬".repeat(before).as_str());
    bar.push_str("🔘");
    bar.push_str("▬".repeat(after).as_str());

    bar
}

pub(super) fn searching_response(query: &str) -> String {
    MessageBuilder::new()
        .push_bold_safe("**🎵 Buscando 🔍**")
        .push_mono_safe(query)
        .build()
}

pub(super) async fn song_added_embed(
    ctx: &Context,
    queue: &Vec<TrackHandle>,
    index: usize,
) -> CreateEmbed {
    let added_track = queue.get(index).unwrap();

    let metadata = added_track.metadata();
    let title = metadata.title.as_ref().unwrap();
    let url = metadata.source_url.as_ref().unwrap();
    let duration = metadata.duration.as_ref().unwrap();
    let thumbnail = metadata.thumbnail.as_ref().unwrap();

    let (requester, channel_name) = get_custom_metadata(ctx, added_track).await;

    let mut embed = CreateEmbed::default();

    embed
        .title(format!("**{title}**"))
        .url(url)
        .color(PRIMARY_COLOR)
        .author(|a| a.name("Encolado").icon_url(requester.face()))
        .thumbnail(thumbnail)
        .field("Canal", channel_name, true)
        .field("Duración", duration_to_minutes(duration), true);

    if index > 0 {
        embed.field("Posición", index, true);

        let time_to_play = queue
            .iter()
            .take(index)
            .fold(Duration::new(0, 0), |acc, track| {
                acc + track.metadata().duration.unwrap()
            });

        embed.field(
            "Tiempo hasta que toque",
            duration_to_minutes(&time_to_play),
            true,
        );
    }

    embed
}

pub(super) fn song_skipped_response(track: &TrackHandle) -> String {
    let metadata = track.metadata();
    let title = metadata.title.as_ref().unwrap();

    MessageBuilder::new()
        .push_bold_safe("⏭️ **Skippeando**: ")
        .push_mono_safe(title)
        .build()
}

pub(super) async fn now_playing_embed(ctx: &Context, track: &TrackHandle) -> CreateEmbed {
    let metadata = track.metadata();
    let title = metadata.title.as_ref().unwrap();
    let url = metadata.source_url.as_ref().unwrap();
    let duration = metadata.duration.as_ref().unwrap();
    let thumbnail = metadata.thumbnail.as_ref().unwrap();

    let (requester, _) = get_custom_metadata(ctx, track).await;

    let mut embed = CreateEmbed::default();

    embed
        .author(|a| {
            a.name("Ahora suena 🎶")
                .icon_url(ctx.cache.current_user().face())
        })
        .title(title)
        .url(url)
        .color(PRIMARY_COLOR)
        .thumbnail(thumbnail);

    let track_position = track.get_info().await.unwrap().position;

    let playing_bar = playing_bar(30, track_position.as_secs_f32() / duration.as_secs_f32());
    let parsed_duration = format!(
        "{} / {}",
        duration_to_minutes(&track_position),
        duration_to_minutes(duration)
    );
    let requester_name = requester.name;

    embed.description(format!(
        "\n`{playing_bar}`\n\n`{parsed_duration}`\n\n**Pedida por:** {requester_name}"
    ));

    embed
}

async fn queue_item(ctx: &Context, track: &TrackHandle) -> String {
    let metadata = track.metadata();
    let title = metadata.title.as_ref().unwrap();
    let url = metadata.source_url.as_ref().unwrap();
    let duration = metadata.duration.as_ref().unwrap();

    let (requester, _) = get_custom_metadata(ctx, track).await;
    let requester_mention = requester.mention();

    MessageBuilder::new()
        .push_named_link_safe(title, url)
        .push(" | ")
        .push_mono_safe(duration_to_minutes(duration))
        .push(" | ")
        .push_bold_safe(format!("Pedida por: {requester_mention}"))
        .build()
}

pub(super) async fn queue_embed(ctx: &Context, queue: &Vec<TrackHandle>) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title("Cola de música").colour(PRIMARY_COLOR);

    let (first, rest) = queue.split_first().unwrap();

    let mut description = MessageBuilder::new();

    description
        .push_underline_line("Tocando:")
        .push_line(queue_item(ctx, first).await);

    if !rest.is_empty() {
        description.push_underline_line("Próximas:");

        for track in rest {
            description.push_line(queue_item(ctx, track).await);
        }
    }

    embed.description(description);

    let count = queue.len();

    let first_duration = first.metadata().duration.unwrap();
    let first_position = first.get_info().await.unwrap().position;
    let remaining_duration = first_duration - first_position;

    let queue_duration = rest.iter().fold(Duration::new(0, 0), |acc, track| {
        acc + track.metadata().duration.unwrap()
    });
    let total_duration = remaining_duration + queue_duration;
    let total_duration = duration_to_minutes(&total_duration);

    embed.footer(|f| {
        f.text(format!(
            "{count} canciones en la cola | Duración total: {total_duration}"
        ))
    });

    embed
}

pub(super) fn song_seeked_response(track: &TrackHandle, position: Duration) -> String {
    let metadata = track.metadata();
    let title = metadata.title.as_ref().unwrap();

    MessageBuilder::new()
        .push_bold_safe("⏩ **Saltando a**: ")
        .push_mono_safe(duration_to_minutes(&position))
        .push(" | ")
        .push_mono_safe(title)
        .build()
}
