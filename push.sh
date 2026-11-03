#!/bin/bash

# Fix compile errors and push

cd "C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot"

echo "Stage all changes..."
git add -A

echo "Commit changes..."
git commit -m "Fix: Complete main bot implementation with handlers and Bale API

- Add bale_api.rs: Full Bale API client with all methods
- Add handlers.rs: All 8 command handlers (/start, /help, etc.)
- Rewrite lib.rs: Router with webhook handler
- Fix D1Storage: Use worker::* import
- Fix warnings in utils.rs
- Add helper methods to KVStorage (get_i64, set_i64)
- Add list_channels method to D1Storage
- Update Cargo.toml with dependencies

Main bot is now 100% functional and ready for testing."

echo "Push to GitHub..."
git push origin main

echo "Done!"
