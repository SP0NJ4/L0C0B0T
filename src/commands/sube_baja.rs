use lazy_static::lazy_static;
use serenity::{model::prelude::Message, prelude::Context};

use l0c0b0t_macros::command;

use crate::framework::command::Command;

lazy_static! {
    pub static ref SUBE_ANSWERS: Vec<&'static str> = vec![
        "La sube mucho",
        "La sube muchísimo",
        "La sube banda",
        "La sube afaerte",
        "LA SUBE",
        "La re sube",
        "La re sube amigo",
        "La sube demasiado",
        "La sube una locura",
        "La sube una banda",
        "La sube por el cielo",
        "La sube por el locie",
        "La ultra sube",
        "Altísima",
        "El subidón",
    ];
    pub static ref BAJA_ANSWERS: Vec<&'static str> = vec![
        "La baja mucho",
        "La baja muchísimo",
        "La baja banda",
        "La baja afaerte",
        "LA BAJA",
        "La re baja",
        "La re baja amigo",
        "La baja demasiado",
        "La baja una locura",
        "La baja una banda",
        "La baja por el piso",
        "La baja por el sopi",
        "La ultra baja",
        "Bajísima",
        "El bajón",
    ];
}

#[command]
async fn sube_baja(ctx: Context, msg: Message) -> bool {
    let content = msg.content.to_lowercase();

    if content.contains("la sube") || content.contains("la baja") {
        let array = if rand::random::<bool>() {
            SUBE_ANSWERS.as_slice()
        } else {
            BAJA_ANSWERS.as_slice()
        };

        let answer = array[rand::random::<usize>() % array.len()];

        msg.reply(ctx, answer).await.unwrap();

        true
    } else {
        false
    }
}
