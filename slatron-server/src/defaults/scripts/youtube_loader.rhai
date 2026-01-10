
// Function signature: fn load_content(params)
// params is a Map with:
// - url: string (Required)

fn load_content(params) {
    if type_of(params) != "map" || !params.contains("url") {
        print("Error: 'url' parameter is required.");
        return [];
    }

    let url = params["url"];
    
    // Use yt-dlp to fetch metadata. 
    // -J: Dump JSON
    // --flat-playlist: If it's a playlist, don't recurse deeply (faster)
    
    let cmd = "yt-dlp";
    let args = ["-J", "--flat-playlist", url];
    
    let result = shell_execute(cmd, args);
    
    if result.code != 0 {
        print("yt-dlp failed: " + result.stderr);
        return [];
    }
    
    let data = parse_json(result.stdout);
    
    // Normalize into a list of items
    let items = [];
    
    if data.contains("entries") {
        // It's a playlist
        for entry in data["entries"] {
             items.push(process_entry(entry));
        }
    } else {
        // Single video
        items.push(process_entry(data));
    }
    
    return items;
}

fn process_entry(entry) {
    let item = #{};
    
    // Extract ID and Title
    let id = entry["id"];
    let title = entry["title"];
    let url = entry["url"]; // In flat-playlist, this is often just the ID
    let webpage_url = entry["webpage_url"];

    // Construct robust URL
    let final_url = "";
    if webpage_url != () {
        final_url = webpage_url;
    } else if url != () {
        // If it looks like an ID (no http), construct youtube link
        if !url.contains("http") {
            final_url = "https://www.youtube.com/watch?v=" + url;
        } else {
            final_url = url;
        }
    } else if id != () {
         final_url = "https://www.youtube.com/watch?v=" + id;
    }
    
    item.title = title;
    item.url = final_url; // Frontend looks for 'url', 'path', or 'content_path'
    
    let duration_seconds = entry["duration"] ?? 0.0;
    // Frontend expects minutes
    item.duration_minutes = duration_seconds / 60.0;
    
    item.metadata = #{
        source: "youtube",
        original_url: final_url,
        uploader: entry["uploader"] ?? ""
    };
    
    return item;
}
