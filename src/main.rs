mod commands;

use clap::Parser;
use log::{error, info, warn};
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{MessageId, ThreadId},
    utils::command::BotCommands,
};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    token: String,

    #[arg(long)]
    chat_id: i64,

    #[arg(long)]
    thread_id: Option<i32>,

    #[arg(long, trailing_var_arg = true)]
    allow_users: Vec<u64>,
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
async fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    info!("Starting bot...");
    info!("Telegram token -> '{}'", cli.token);
    info!("Target chat ID -> '{}'", cli.chat_id);
    if let Some(thread_id) = cli.thread_id {
        info!("Target thread ID -> '{}'", thread_id);
    } else {
        info!("Target thread ID -> 'None'");
    }
    if cli.allow_users.is_empty() {
        info!("Alowed users -> 'Empty'");
        warn!("FORWARDING MESSAGES TO ALL USERS IS ALLOWED!!!");
    } else {
        info!("Alowed users -> '{:?}'", cli.allow_users);
    }

    let bot = Bot::new(cli.token);
    info!("Bot instance created");

    let allow_users = Arc::new(cli.allow_users);
    let target_chat_id = Arc::new(ChatId(cli.chat_id));
    let thread_id = if let Some(thread_id) = cli.thread_id {
        Some(ThreadId(MessageId(thread_id)))
    } else {
        None
    };

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(commands::start))
        .branch(dptree::case![Command::GetChatId].endpoint(commands::get_chat_id));

    let handler = {
        let target_chat_id = Arc::clone(&target_chat_id);
        Update::filter_message().branch(command_handler).branch(
            dptree::filter(|msg: Message| msg.chat.is_private()).endpoint(
                move |bot: Bot, msg: Message| {
                    let allow_users = Arc::clone(&allow_users);
                    let chat_id = Arc::clone(&target_chat_id);
                    async move {
                        commands::forward(bot, msg, allow_users, chat_id, thread_id).await
                    }
                },
            ),
        ).branch(dptree::endpoint(|| async move { respond(()) }))
    };

    info!("Check bot is present in chat");
    check_bot_in_chat(&bot, target_chat_id).await;

    info!("Staring dispatcher...");
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn check_bot_in_chat(bot: &Bot, chat_id: Arc<ChatId>) {
    let bot_id = bot.get_me().await.unwrap().id;

    if let Ok(user) = bot.get_chat_member(*chat_id, bot_id).await
        && user.is_present()
    {
        info!("Bot has access to chat");
        bot.send_message(*chat_id, "Bot initialized\nStating dispatcher..")
            .await
            .unwrap();
    } else {
        error!("Bot cannot access the chat");
        panic!("Bot cannot access the chat");
    }
}
