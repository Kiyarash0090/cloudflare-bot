pub mod bale_api;
pub mod handlers;

use worker::*;
use shared::{BaleUpdate, D1Storage, KVStorage};
use bale_api::BaleBot;
use handlers::CommandHandler;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Setup logging
    console_error_panic_hook::set_once();

    // Router
    let router = Router::new();
    
    router
        .get("/", |_, _| Response::ok("Telegram-Bale Bot is running!"))
        .get("/health", |_, _| Response::ok("OK"))
        .post_async("/bale/webhook", |mut req, ctx| async move {
            handle_bale_webhook(req, ctx).await
        })
        .run(req, env)
        .await
}

async fn handle_bale_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Parse update
    let update: BaleUpdate = match req.json().await {
        Ok(u) => u,
        Err(e) => {
            console_log!("Failed to parse update: {}", e);
            return Response::error("Bad request", 400);
        }
    };

    // Get environment variables
    let bale_token = ctx.env.secret("BALE_TOKEN")?.to_string();
    let admin_user_id: i64 = ctx.var("ADMIN_USER_ID")?.to_string().parse()
        .map_err(|_| worker::Error::RustError("Invalid ADMIN_USER_ID".to_string()))?;

    // Get D1 and KV
    let d1 = ctx.env.d1("DB")?;
    let kv = ctx.env.kv("KV")?;

    // Create handler
    let bale = BaleBot::new(bale_token, admin_user_id);
    let d1_storage = D1Storage::new(d1);
    let kv_storage = KVStorage::new(kv);
    
    let handler = CommandHandler::new(bale, d1_storage, kv_storage, admin_user_id);

    // Handle update in background
    ctx.wait_until(async move {
        if let Err(e) = handler.handle_update(update).await {
            console_log!("Error handling update: {}", e);
        }
    });

    // Return 200 immediately
    Response::ok("OK")
}
