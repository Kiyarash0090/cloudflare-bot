use crate::Result;

/// Parse Telegram channel identifier
/// Supports: @username, numeric ID, or https://t.me/username
pub fn parse_channel_identifier(input: &str) -> Result<String> {
    let input = input.trim();
    
    // Handle t.me links
    if input.starts_with("https://t.me/") || input.starts_with("http://t.me/") {
        if let Some(username) = input.split('/').last() {
            return Ok(username.trim_start_matches('@').to_string());
        }
    }
    
    // Handle @username or numeric ID
    Ok(input.trim_start_matches('@').to_string())
}

/// Format timestamp to readable string
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::DateTime;
    
    let datetime = DateTime::from_timestamp(timestamp, 0);
    match datetime {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "Invalid timestamp".to_string(),
    }
}

/// Check if user is admin
pub fn is_admin(user_id: i64, admin_id: i64) -> bool {
    user_id == admin_id
}

/// Validate file size based on channel settings
pub fn validate_file_size(file_size: i64, max_size_mb: i32) -> bool {
    let max_bytes = (max_size_mb as i64) * 1024 * 1024;
    file_size <= max_bytes
}

/// Format file size to human readable
pub fn format_file_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Escape markdown special characters
pub fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_channel_identifier() {
        assert_eq!(
            parse_channel_identifier("@username").unwrap(),
            "username"
        );
        assert_eq!(
            parse_channel_identifier("username").unwrap(),
            "username"
        );
        assert_eq!(
            parse_channel_identifier("https://t.me/username").unwrap(),
            "username"
        );
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
