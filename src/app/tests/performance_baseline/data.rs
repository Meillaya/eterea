pub(crate) fn generated_archive(count: usize) -> String {
    let mut out = String::from("[");
    for i in 0..count {
        if i > 0 {
            out.push(',');
        }
        let handle = format!("author{}", i % 25);
        let tag = match i % 5 {
            0 => "rust",
            1 => "design",
            2 => "systems",
            3 => "performance",
            _ => "ai",
        };
        let media = if i % 7 == 0 {
            format!(
                r#","extended_media":[{{"media_url_https":"https://pbs.twimg.com/media/{i}.jpg"}}]"#
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            r#"{{"screen_name":"{handle}","name":"Author {handle}","full_text":"Generated bookmark {i} about #{tag} and local-first archives","tweeted_at":"2024-05-{day:02}T12:00:00Z","tweet_url":"https://x.com/{handle}/status/{i}"{media}}}"#,
            day = (i % 28) + 1,
        ));
    }
    out.push(']');
    out
}

pub(crate) fn generated_author_heavy_archive(count: usize) -> String {
    let mut out = String::from("[");
    for i in 0..count {
        if i > 0 {
            out.push(',');
        }
        let handle = format!("unique_author_{i:05}");
        out.push_str(&format!(
            r#"{{"screen_name":"{handle}","name":"Author {i}","full_text":"Generated high-cardinality author bookmark {i} about directory performance","tweeted_at":"2024-06-{day:02}T12:00:00Z","tweet_url":"https://x.com/{handle}/status/{i}"}}"#,
            day = (i % 28) + 1,
        ));
    }
    out.push(']');
    out
}
