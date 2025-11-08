use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use teloxide::{
    payloads::ForwardMessageSetters, prelude::*, sugar::request::RequestReplyExt, types::ThreadId,
};

const START_TEXT: &str = "Hi.";

pub async fn start(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    bot.send_message(msg.chat.id, START_TEXT)
        .reply_to(msg.id)
        .await?;
    Ok(())
}

pub async fn get_chat_id(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    let thread_id = match msg.thread_id {
        Some(value) => value.0.0.to_string(),
        None => String::from("None"),
    };

    bot.send_message(
        msg.chat.id,
        format!("Chat ID: {}, Thread ID: {}", msg.chat.id, thread_id),
    )
    .reply_to(msg.id)
    .await?;
    Ok(())
}

pub async fn forward(
    bot: Bot,
    msg: Message,
    allow_users: Arc<Option<Vec<u64>>>,
    target_chat_id: Arc<ChatId>,
    thread_id: Option<ThreadId>,
) -> Result<(), teloxide::RequestError> {
    let Some(user) = msg.from else {
        bot.send_message(msg.chat.id, "⚠️ Failed get your ID")
            .await?;
        error!(
            "Failed get user ID. Chat ID: '{}', Message ID: '{}'",
            msg.chat.id, msg.id
        );
        return Ok(());
    };

    let is_allow = match allow_users.as_deref() {
        Some(value) => value.contains(&user.id.0),
        None => true,
    };

    if !is_allow {
        bot.send_message(msg.chat.id, "⛔️ Access Denied").await?;
        warn!(
            "Access denied for user '{}' trying to forward message '{}",
            user.id, msg.id
        );
        return Ok(());
    }

    let mut message = bot.forward_message(*target_chat_id, msg.chat.id, msg.id);
    if let Some(thread_id) = thread_id {
        message = message.message_thread_id(thread_id);
    }

    message.await?;

    bot.send_message(msg.chat.id, "✅ Success").await?;
    info!(
        "Message '{}' forwarded to '{}' from user '{}'",
        msg.id, target_chat_id.0, user.id
    );
    Ok(())
}
