-- Default admin user (password: admin)
-- Generated with: bcrypt::hash("admin", 12)
INSERT INTO users (username, password_hash, role) VALUES
('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYqEz0xLQK2', 'admin');

-- Global settings
INSERT INTO global_settings (key, value, description) VALUES
('content_error_behavior', 'skip', 'skip | retry | error_screen | previous'),
('content_error_retry_attempts', '3', 'Retry attempts on content error'),
('content_error_retry_delay_secs', '5', 'Delay between retries'),
('node_heartbeat_timeout_secs', '15', 'Mark node offline after N seconds'),
('default_transition_type', 'cut', 'cut | fade');

-- Builtin Scripts
INSERT INTO scripts (name, description, script_type, script_content, parameters_schema, is_builtin) VALUES
(
    'yt-dlp Video Downloader',
    'Download videos from YouTube and other sites using yt-dlp',
    'content_loader',
    'let url = params["url"];
let output_dir = params["output_dir"];
let quality = params["quality"];

let cmd = `yt-dlp -f "bestvideo[height<=${quality}]+bestaudio" -o "${output_dir}/%(title)s.%(ext)s" "${url}"`;
shell_execute(cmd);
`${output_dir}/video.mp4`',
    '{"type":"object","properties":{"url":{"type":"string","title":"Video URL"},"output_dir":{"type":"string","title":"Output Directory"},"quality":{"type":"string","title":"Quality","enum":["best","1080","720","480"],"default":"720"}},"required":["url","output_dir"]}',
    TRUE
),
(
    'Station Bug Overlay',
    'Add a persistent logo overlay (station bug)',
    'overlay',
    'let logo_path = params["logo_path"];
let position = params["position"];
let opacity = params["opacity"];

let coords = if position == "top-left" {
    #{x: 10, y: 10}
} else if position == "top-right" {
    #{x: get_video_width() - 110, y: 10}
} else if position == "bottom-left" {
    #{x: 10, y: get_video_height() - 110}
} else {
    #{x: get_video_width() - 110, y: get_video_height() - 110}
};

mpv_overlay(logo_path, coords.x, coords.y, opacity);',
    '{"type":"object","properties":{"logo_path":{"type":"string","title":"Logo File Path"},"position":{"type":"string","title":"Position","enum":["top-left","top-right","bottom-left","bottom-right"],"default":"top-right"},"opacity":{"type":"number","title":"Opacity","minimum":0,"maximum":1,"default":0.8}},"required":["logo_path"]}',
    TRUE
),
(
    'Auto-Loop Short Content',
    'Automatically loop content shorter than the time block',
    'global',
    'let content_duration = get_content_duration();
let block_duration = get_block_duration();

if content_duration < block_duration {
    mpv_set_loop(true);
}',
    '{"type":"object","properties":{}}',
    TRUE
),
(
    'No Signal Pattern',
    'Display test pattern when no content is scheduled',
    'content_loader',
    'mpv_play("/builtin/test-pattern.mp4");
mpv_set_loop(true);
"/builtin/test-pattern.mp4"',
    '{"type":"object","properties":{}}',
    TRUE
);
