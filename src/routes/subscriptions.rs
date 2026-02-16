use actix_web::{
    HttpResponse,
    web::{self},
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

/// `subscribe` orchestrates the work to be done by calling the required routines and translates their out-come into the prosper response according to the rules and conventions of HTTP
// #[tracing::instrument] creates a span at the beginning of the function invocation and automatically attaches all arguments passed to the function to the context of the span - in our case, form and pool.
#[tracing::instrument(
    // name can be used to specify the message associated to the function span - if omitted, it defaults to the function name
    name = "Adding a new subscriber",
    // we can explicitly tell tracing what to ignore using the skip directive.
    // form will be ignored because we will unwrap it inside the fields()
    // and pool will be ignored because its not easily displayable
    skip(form, pool),
    // We can also enrich the span’s context using the fields directive. It leverages the same syntax we have already seen for the info_span! macro.
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match NewSubscriber::try_from(form.0) {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// insert_subscriber takes care of the database logic.
/// It has no awareness of the sorrounding web framework, that means, no web::Form or web::Data wrappers as inputs types
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ( $1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
