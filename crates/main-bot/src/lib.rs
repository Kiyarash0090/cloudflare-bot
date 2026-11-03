mod bale_api;
mod handlers;
mod session;

use bale_api::BaleApi;
use handlers::BotHandler;
use shared::{D1Storage, KVStorage};
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Setup logging
    console_error_panic_hook::set_once();

    // Get router
    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("Telegram-Bale Bot is running!"))
        .post_async("/bale/webhook", |req, ctx| async move {
            handle_webhook(req, ctx).await
        })
        .get("/health", |_, _| {
            Response::from_json(&serde_json::json!({
                "status": "ok",
                "service": "telegram-bale-bot"
            }))
        })
        .get("/session", |_, _| session::get_session_ui())
        .post_async("/session/save", session::save_session)
        .get_async("/session/get", session::get_session)
        .run(req, env)
        .await
}

async fn handle_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Get environment variables
    let bale_token = ctx
        .secret("BALE_TOKEN")?
        .to_string();

    let admin_user_id: i64 = ctx
        .var("ADMIN_USER_ID")?
        .to_string()
        .parse()
        .unwrap_or(0);

    // Initialize storage
    let db = ctx.env.d1("DB")?;
    let kv = ctx.env.kv("KV")?;

    let storage = D1Storage::new(db);
    let kv_storage = KVStorage::new(kv);

    // Initialize Bale API
    let api = BaleApi::new(bale_token);

    // Initialize handler
    let handler = BotHandler::new(api, storage, kv_storage, admin_user_id);

    // Parse update
    let update: bale_api::BaleUpdate = match req.json().await {
        Ok(u) => u,
        Err(e) => {
            console_log!("Failed to parse update: {}", e);
            return Response::error("Bad Request", 400);
        }
    };

    // Handle update
    match handler.handle_update(update).await {
        Ok(_) => Response::ok("OK"),
        Err(e) => {
            console_log!("Error handling update: {}", e);
            Response::ok("OK") // Always return OK to Bale
        }
    }
}
