use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
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
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// insert_subscriber takes care of the database logic.
/// It has no awareness of the sorrounding web framework, that means, no web::Form or web::Data wrappers as inputs types
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ( $1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
