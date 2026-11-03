// Telegram Poller Durable Object
// TODO: Implement

pub mod poller;
pub mod message_handler;

use worker::*;

#[durable_object]
pub struct TelegramPoller {
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for TelegramPoller {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let url = req.url()?;
        
        match url.path() {
            "/poll" => {
                // TODO: Implement polling logic
                Response::ok("Polling started")
            }
            _ => Response::error("Not found", 404),
        }
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/", |_, _| Response::ok("Telegram Poller is running!"))
        .run(req, env)
        .await
}
