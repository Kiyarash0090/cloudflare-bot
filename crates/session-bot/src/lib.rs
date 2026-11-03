// Placeholder for session bot implementation
// This will be implemented in the next phase

pub mod telegram_auth;
pub mod handlers;

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();
    
    router
        .get("/", |_, _| Response::ok("Session Bot is running!"))
        .post_async("/session-bot/webhook", |mut req, ctx| async move {
            // TODO: Implement session bot webhook handler
            Response::ok("OK")
        })
        .run(req, env)
        .await
}
