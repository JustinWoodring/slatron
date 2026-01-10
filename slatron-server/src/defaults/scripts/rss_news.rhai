// Parameters: { "url": "https://..." }
let url = params.get("url");

if url != () {
    log_info("Fetching RSS feed from: " + url);
    let xml_str = http_get(url);
    
    // Check for error prefix from http_get
    // Helper to extract text from XML node (which might be a Map with $text or $value)
    let extract_text = |v| {
        if type_of(v) == "map" {
             let t = v.get("$text");
             if t != () { return t; }
             let val = v.get("$value");
             if val != () { return val; }
             return v; // Fallback
        }
        return v;
    };

    if xml_str.contains("Error fetching URL") {
        print("RSS Fetch Error: " + xml_str);
    } else {
        let data = parse_xml(xml_str);
        
        // Handle common RSS structure
        let channel = ();
        if data.contains("rss") {
            channel = data.get("rss").get("channel");
        } else if data.contains("channel") {
            channel = data.get("channel");
        }
        
        if channel != () {
            let title = extract_text.call(channel.get("title"));
            if title == () { title = "News Feed"; }
            
            context += "\n## Latest Headlines from " + title + ":\n";
            context += "(Use these current events for banter/commentary)\n";
            
            let items = channel.get("item");
            if items != () {
                let list = [];
                if type_of(items) == "array" {
                    list = items;
                } else {
                    list.push(items);
                }
                
                // Process up to 10 items
                let count = list.len();
                if count > 10 { count = 10; }
                
                for i in 0..count {
                   let item = list[i];
                   
                   let i_title = extract_text.call(item.get("title"));
                   if i_title == () { continue; }
                   
                   // "Next Sibling" Lookahead
                   let next_title = "";
                   if i + 1 < count {
                       let next_item = list[i+1];
                       if next_item != () {
                           let nt = extract_text.call(next_item.get("title"));
                           if nt != () { next_title = nt; }
                       }
                   }
                   
                   let i_desc = extract_text.call(item.get("description"));
                   let i_date = extract_text.call(item.get("pubDate"));
                   let i_cats = item.get("category"); // Might be array or map
                   
                   // Format Categories
                   let cat_str = "";
                   if i_cats != () {
                       if type_of(i_cats) == "array" {
                           for c in i_cats {
                               let c_text = extract_text.call(c);
                               if cat_str != "" { cat_str += ", "; }
                               cat_str += c_text;
                           }
                       } else {
                           cat_str = extract_text.call(i_cats);
                       }
                   }
                   
                   context += "\n### " + i_title;
                   if i_date != () { context += " (" + i_date + ")"; }
                   context += "\n";
                   
                   if i_desc != () { 
                       context += i_desc + "\n"; 
                   }
                   if cat_str != "" {
                       context += "Tags: " + cat_str + "\n";
                   }
                }
            } else {
                context += "No items found in feed.\n";
            }
        } else {
            context += "Error: Invalid RSS Feed format (missing channel).\n";
        }
    }
} else {
    log_info("RSS News Script: No 'url' parameter provided.");
}
