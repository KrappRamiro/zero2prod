use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

// An extension trait to provide the `graphemes` method
// on `String` and `&str`
use unicode_segmentation::UnicodeSegmentation;

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
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().body(format!("{} is not a valid name", &form.name));
    }
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

/// Returns true if the input satisfies all our validation constraints
/// on subscriber names, `false` otherwise
pub fn is_valid_name(s: &str) -> bool {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character
    let is_empty_or_whitespace = s.trim().is_empty();

    // A grapheme is defined by the Unicode standart as a "user-perceived"
    // character: `å` is a single grapheme, but it is composed of two characters
    // (`a` and `̊``).
    //
    // `graphemes` returns an iterator over the graphemes in the input `s`.
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    let is_too_long = s.graphemes(true).count() > 256;

    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|char| forbidden_characters.contains(&char));

    // Return `false` if any of our conditions have been violated
    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}
