// Station Bug
fn transform(settings) {
    return settings;
}

// Called when playback starts
fn on_load(settings) {
    let logo_path = download_file(settings["station_bug_image"], "~/station-logo");
    
    // 2. Construct the FFmpeg filter string
    // "movie='path'[logo];[vid][logo]overlay=X:Y"
    // X = main_w - overlay_w - 20 (20px from right)
    // Y = 20 (20px from top)
    let filter = "movie='" + logo_path + "'[logo];[logo]scale=20:-1[logo_scaled];[vid][logo_scaled]overlay=main_w-overlay_w-20:20";
    
    // 3. Send command to MPV to add the filter with a specific label (@bug)
    let cmd = ["vf", "add", "@bug:lavfi=[" + filter + "]"];
    
    mpv_send(#{ command: cmd });
}

// Called when playback ends or changes
fn on_unload() {
    // 4. Remove the filter by its label
    let cmd = ["vf", "remove", "@bug"];
    mpv_send(#{ command: cmd });
}
