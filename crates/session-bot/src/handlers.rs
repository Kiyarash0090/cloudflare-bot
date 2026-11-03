// Session bot command handlers
// TODO: Implement

use shared::Result;

pub struct SessionHandler {
    // Will be implemented
}

impl SessionHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle_start(&self) -> Result<()> {
        Ok(())
    }

    pub async fn handle_login(&self) -> Result<()> {
        Ok(())
    }

    pub async fn handle_code(&self) -> Result<()> {
        Ok(())
    }

    pub async fn handle_password(&self) -> Result<()> {
        Ok(())
    }
}
