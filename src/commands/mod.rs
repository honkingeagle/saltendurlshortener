use crate::SharedState;
use nanoid::nanoid;
use std::error::Error;
use teloxide::prelude::*;
use url::Url;

type BotResult = Result<(), Box<dyn Error + Send + Sync>>;

async fn answer(bot: Bot, msg: Message, state: SharedState) -> BotResult {
    match msg.text() {
        Some(text) => {
            match Url::parse(text) {
                Ok(_) => {
                    let id = nanoid!(10);
                    let prod_url = &state.prod_url;
                    let generated_url = format!("{prod_url}/{id}");

                    let query = sqlx::query("INSERT INTO urls (real_url, generated_url) VALUES ($1, $2)")
                        .bind(text)
                        .bind(&generated_url)
                        .execute(&state.pool)
                        .await;

                    match query {
                        Ok(_) => {
                            bot.send_message(msg.chat.id, format!("Here's your shortened url {generated_url}")).await?;
                        },
                        Err(_) => {
                            bot.send_message(msg.chat.id, format!("Unable to shorten your url. Try again!")).await?;
                        }
                        
                    }

                },
                Err(_) => {
                    bot.send_message(msg.chat.id, format!("Send me a valid url"))
                        .await?;
                }
            }
        },
        None => {
            bot.send_message(msg.chat.id, "Send me a url").await?;
        }
    };
    Ok(())
}


pub async fn run(state: SharedState) {
    let bot = Bot::new(&state.teloxide_token);

    Dispatcher::builder(bot, Update::filter_message().endpoint(answer))
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
