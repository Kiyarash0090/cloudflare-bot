pub mod bale_api;
pub mod handlers;

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::ok("Telegram-Bale Bot is running!"))
        .get("/health", |_, _| Response::ok("OK"))
        .run(req, env)
        .await
}
