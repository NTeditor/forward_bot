use log::{error, info, warn};
use std::sync::Arc;
use teloxide::{prelude::*, sugar::request::RequestReplyExt};

const START_TEXT: &str = "Hi.";

pub async fn start(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    bot.send_message(msg.chat.id, START_TEXT).await?;
    Ok(())
}

pub async fn get_chat_id(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    let thread_id = if let Some(thread_id) = msg.thread_id {
        thread_id.0.0.to_string()
    } else {
        String::from("None")
    };
    bot.send_message(
        msg.chat.id,
        format!("Chat id: {}, Thread: {}", msg.chat.id, thread_id),
    )
    .reply_to(msg.id)
    .await?;

    Ok(())
}

pub async fn forward(
    bot: Bot,
    msg: Message,
    allow_users: Arc<Vec<u64>>,
    target_chat_id: Arc<ChatId>,
) -> Result<(), teloxide::RequestError> {
    if let Some(user) = msg.from {
        if allow_users.is_empty() {
            warn!("allow_users is empty: ALLOW FORWARD FOR ALL USERS!!!");
        }
        if allow_users.is_empty() || allow_users.contains(&user.id.0) {
            bot.forward_message(*target_chat_id, msg.chat.id, msg.id)
                .await?;
            bot.send_message(msg.chat.id, "✅ Success").await?;
            info!(
                "Message '{}' forwarded to '{}' from user '{}'",
                msg.id, target_chat_id.0, user.id
            );
        } else {
            bot.send_message(msg.chat.id, "⛔️ Access Denied").await?;
            warn!(
                "Access denied for user '{}' trying to forward message '{}",
                user.id, msg.id
            );
        }
    } else {
        bot.send_message(msg.chat.id, "⚠️ Failed get your id")
            .await?;
        error!(
            "Failed get user id. Chat: '{}', Message: '{}'",
            msg.chat.id, msg.id
        );
    }
    Ok(())
}
