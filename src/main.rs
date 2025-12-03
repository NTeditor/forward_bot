mod config;
mod handler;

use std::sync::Arc;

use anyhow::{Context, Result};
use config::Config;
use handler::*;
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree,
    macros::BotCommands,
    prelude::Dispatcher,
    types::{Message, Update},
};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, time::ChronoUtc},
    prelude::*,
};

const CONFIG_PATH: Option<&str> = option_env!("CONFIG_PATH");

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "snake_case",
    description = "Send message to forward to private chat.\nThese commands are supported:"
)]
enum Command {
    #[command(description = "Print start message.")]
    Start,
    #[command(description = "Print this text.")]
    Help,
    #[command(description = "Print chat_id and thread_id.")]
    GetChatId,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let config_path = CONFIG_PATH.unwrap_or("config.toml");
    let config = Config::load(config_path).context("Failed load config")?;
    let config = Arc::new(config);
    let bot = Bot::new(&config.token);

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .branch(teloxide::filter_command::<Command, _>().endpoint(command_handler))
            .branch(
                dptree::filter(|message: Message| message.chat.is_private())
                    .endpoint(forward_handler),
            ),
    )
    .dependencies(dptree::deps![config])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}

fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let timer = ChronoUtc::new("%H:%M:%S".to_string());

    let fmt_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(true)
        .with_timer(timer)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    tracing::info!("Logger is initialized");
}
