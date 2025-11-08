mod commands;
mod config;

use anyhow::{Result, bail};
use clap::Parser;
use log::{info, warn};
use std::{path::PathBuf, sync::Arc};
use teloxide::{
    prelude::*,
    types::{MessageId, ThreadId},
    utils::command::BotCommands,
};

use crate::config::Config;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    config: PathBuf,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Start bot")]
    Start,
    #[command(description = "get chat id")]
    GetChatId,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let cli = Cli::parse();
    info!("Starting bot..");
    info!("Reading config..");
    let config = Config::read_from_file(&cli.config)?;
    info!("{:#?}", config);

    if config.allowed_users.is_none() {
        warn!("FORWARDING MESSAGES TO ALL USERS IS ALLOWED!!!");
    }

    let bot = Bot::new(config.token);
    info!("Bot instance created");

    let allowed_users = Arc::new(config.allowed_users);
    let chat_id = Arc::new(ChatId(config.chat_id));
    let thread_id = if let Some(thread_id) = config.thread_id {
        Some(ThreadId(MessageId(thread_id)))
    } else {
        None
    };

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(commands::start))
        .branch(dptree::case![Command::GetChatId].endpoint(commands::get_chat_id));

    let handler = {
        let chat_id = chat_id.clone();
        Update::filter_message()
            .branch(command_handler)
            .branch(
                dptree::filter(|msg: Message| msg.chat.is_private()).endpoint(
                    move |bot: Bot, msg: Message| {
                        let allowed_users = allowed_users.clone();
                        let chat_id = chat_id.clone();
                        async move {
                            commands::forward(bot, msg, allowed_users, chat_id, thread_id).await
                        }
                    },
                ),
            )
            .branch(dptree::endpoint(|| async move { respond(()) }))
    };

    info!("Checking if a bot is present in a chat");
    bot_is_present(&bot, chat_id).await?;

    info!("Staring dispatcher...");
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn bot_is_present(bot: &Bot, chat_id: Arc<ChatId>) -> Result<()> {
    let bot_id = bot.get_me().await?.id;
    match bot.get_chat_member(*chat_id, bot_id).await {
        Ok(user) if user.is_present() => {
            info!("Bot has access to chat");
            bot.send_message(*chat_id, "Bot initialized\nStating dispatcher..")
                .await?;
        }
        Ok(_) => {
            bail!("Bot is not present in the chat");
        }
        Err(e) => {
            bail!("Failed to check bot presence: {}", e);
        }
    }
    Ok(())
}
