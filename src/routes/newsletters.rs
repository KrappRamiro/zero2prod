use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    domain::SubscriberEmail,
    email_client::{self, EmailClient},
    routes::error_chain_fmt,
};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

// Same logic to get the full error chain on `Debug`
impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for sub in subscribers {
        // Why use a Result? Well, the rules of what a "valid" email is can change overtime, so
        // what once was a valid email, in the future might raise an error when validating the
        // email string
        match sub {
            Ok(sub) => {
                email_client
                    .send_email(
                        &sub.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| format!("Failed to send newsletter issue to {}", sub.email))?;
            }
            Err(error) => {
                tracing::warn!(
                    // We record the error chain as a structured field
                    // on the log record.
                    error.cause_chain = ?error,
                    // Using `\` to split a long string literal over
                    // two lines, without creating a `\n` character.
                    "Skipping a confirmed subscriber. \
                Their stored contact details are invalid",
                )
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
/// We return a `Vec` of `Result`'s in the happy case.
/// This allows the caller to bubble up error due to network issues or other transient failures
/// using the `?` operator, while the compiler forces them to handle the subtler mapping error.
/// See http://sled.rs/errors.html for a deep-dive about this technique.
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subs = sqlx::query!(
        r#"
        SELECT EMAIL
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| match SubscriberEmail::parse(row.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(err) => Err(anyhow::anyhow!(err)),
    })
    .collect();

    Ok(confirmed_subs)
}
