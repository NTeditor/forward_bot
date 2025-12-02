use crate::{Command, config::Config};
use anyhow::{Context, Result, bail};
use std::sync::Arc;
use teloxide::{
    Bot,
    payloads::{ForwardMessageSetters, SendMessageSetters},
    prelude::Requester,
    sugar::request::RequestReplyExt,
    types::{ChatId, Message, MessageId, ParseMode, ThreadId},
    utils::command::BotCommands,
};

pub async fn command_handler(
    bot: Bot,
    message: Message,
    command: Command,
    config: Arc<Config>,
) -> Result<()> {
    tracing::info!(
        command = ?command,
        chat_id = ?message.chat.id,
        user_id = ?message.from.map(|u| u.id),
        "Received and processing command"
    );

    match command {
        Command::Start => {
            bot.send_message(message.chat.id, &config.messages.start_command)
                .await
        }
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await
        }
        Command::GetChatId => {
            let message_text = format!(
                "chat_id = <code>{}</code>\nthread_id = <code>{:?}</code>",
                message.chat.id.0,
                message.thread_id.map(|id| id.0.0)
            );
            bot.send_message(message.chat.id, message_text)
                .reply_to(message.id)
                .parse_mode(ParseMode::Html)
                .await
        }
    }
    .with_context(|| format!("Failed to send response for command: {:?}", command))?;

    Ok(())
}

pub async fn forward_handler(bot: Bot, message: Message, config: Arc<Config>) -> Result<()> {
    tracing::info!(
        chat_id = ?message.chat.id,
        message_id = ?message.id,
        "Forward message (user identification)"
    );

    let Some(from) = message.from else {
        tracing::error!(
            chat_id = ?message.chat.id,
            message_id = ?message.id,
            "The message sender could not be determined"
        );
        bot.send_message(
            message.chat.id,
            "ERROR: The message sender could not be determined",
        )
        .reply_to(message.id)
        .await
        .context("Failed to send 'error' message")?;

        bail!("The message sender could not be determined");
    };

    tracing::info!(
        username = ?from.username,
        user_id = ?from.id,
        "The user has been successfully identified"
    );

    if let Some(allowed_users) = &config.allowed_users
        && !allowed_users.contains(&from.id.0)
    {
        tracing::info!(
            username = ?from.username,
            user_id = ?from.id,
            "Access denied for forwarding message"
        );
        bot.send_message(message.chat.id, &config.messages.access_denied_forward)
            .reply_to(message.id)
            .await
            .context("Failed to send 'access denied' message")?;
        return Ok(());
    } else {
        tracing::info!(
            username = ?from.username,
            user_id = ?from.id,
            "Access granted for forwarding message"
        );
    }

    let result = bot
        .forward_message(ChatId(config.target.chat_id), message.chat.id, message.id)
        .message_thread_id(ThreadId(MessageId(config.target.thread_id)))
        .await;

    match result {
        Ok(_) => {
            tracing::info!(
                chat_id = ?message.chat.id,
                message_id = ?message.id,
                user_id = ?from.id,
                username = ?from.username,
                "Success forward message"
            );
            bot.send_message(message.chat.id, &config.messages.success_forward)
                .reply_to(message.id)
                .await
                .context("Failed to send 'success' message")?;
        }
        Err(err) => {
            tracing::error!(
                chat_id = ?message.chat.id,
                message_id = ?message.id,
                username = ?from.username,
                user_id = ?from.id,
                err = ?err,
                "Failed to forward message"
            );
            bot.send_message(message.chat.id, format!("Failed to forward message"))
                .reply_to(message.id)
                .await
                .context("Failed to send 'error' message")?;
            return Ok(());
        }
    }
    Ok(())
}
