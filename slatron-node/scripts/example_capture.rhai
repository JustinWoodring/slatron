
fn transform(settings) {
    let url = "https://www.google.com";
    let output_path = "/tmp/slatron_capture_google.png";

    print("Capturing website: " + url);
    let result = capture_website(url, output_path);

    if result.sub_string(0, 5) == "Error" {
        print("Capture failed: " + result);
    } else {
        print("Capture successful: " + result);
        // Override the playback path to the captured screenshot
        settings.set_path(result);
        // OR: settings.path = result;
    }

    // Set duration for the slide
    settings.set_loop(false); // Image, so maybe no loop or loop for duration?
    // MPV usually handles images by displaying them indefinitely unless duration is set
    // But slatron main loop logic might restart it?
    // Let's just set the path.

    return settings;
}
