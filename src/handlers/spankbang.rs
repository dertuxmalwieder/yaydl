/*
 * The contents of this file are subject to the terms of the
 * Common Development and Distribution License, Version 1.0 only
 * (the "License").  You may not use this file except in compliance
 * with the License.
 *
 * See the file LICENSE in this distribution for details.
 * A copy of the CDDL is also available via the Internet at
 * http://www.opensource.org/licenses/cddl1.txt
 *
 * When distributing Covered Code, include this CDDL HEADER in each
 * file and include the contents of the LICENSE file from this
 * distribution.
 */

// specific url path format for this site
// https://spankbang.com/5-char-id/video/description+seprated+by+plus+char
//
// example:  https://spankbang.com/12345/video/description+for+this+video
// base filename: description_for_this_video lenght maximum is 142
// filename: description_for_this_video-12345.mp4
//
// https://spankbang.com/70841/video/nikki+fritz
//

// Yet Another Youtube Down Loader
// - Spankbang handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use fantoccini::ClientBuilder;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::runtime;
use url::Url;

const MAX_FILENAME_LENGTH: usize = 142; // filename is based on url path description string

static mut VIDEO_INFO: String = String::new();

unsafe fn get_video_info(url: &str, webdriver_port: u16) -> Result<Html> {
    if VIDEO_INFO.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let local_url = url.to_owned();

        let rt = runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        rt.block_on(async move {
            let webdriver_url = format!("http://localhost:{}", webdriver_port);
            let c = ClientBuilder::native()
                .connect(&webdriver_url)
                .await
                .expect("failed to connect to web driver");
            c.goto(&local_url).await.expect("could not go to the URL");
            let body = c.source().await.expect("could not read the site source");
            c.close_window().await.expect("could not close the window");

            VIDEO_INFO = body;
        });
    }

    // Return it:
    let d = Html::parse_document(&VIDEO_INFO);
    Ok(d)
}

// Implement the site definition:
struct SpankbangHandler;
impl SiteDefinition for SpankbangHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"spankbang.com/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // Generic has playlists.
        Ok(false)
    }

    fn find_video_title<'a>(&'a self, url: &'a str, _webdriver_port: u16) -> Result<String> {
        // generates a valid base filename from url path for linux and windows
        // video title is less reliable to generate base filename for this particular site
        Ok(url_filename(url.to_string()))
    }

    fn find_video_direct_url<'a>(
        &'a self,
        url: &'a str,
        webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        unsafe {
            let video_info = get_video_info(url, webdriver_port)?;

            let url_selector = Selector::parse(r#"source[type="video/mp4"]"#).unwrap();
            let url_elem = video_info.select(&url_selector).next().unwrap();
            let url_contents = url_elem.value().attr("src").unwrap();

            Ok(url_contents.to_string())
        }
    }

    fn does_video_exist<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<bool> {
        unsafe {
            let _video_info = get_video_info(url, webdriver_port);
            Ok(!VIDEO_INFO.is_empty())
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "Spankbang".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        _url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        Ok("mp4".to_string())
    }

    fn web_driver_required<'a>(&'a self) -> bool {
        true
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &SpankbangHandler as &dyn SiteDefinition
}

// covert url path to base filename
// path pattern is specific to web sites
fn url_filename(url: String) -> String {
    // https://spankbang.com/12345/video/description+for+this+video
    // extract url path
    let path = match Url::parse(&url) {
        Ok(urlx) => urlx.path().to_string(),
        Err(e) => {
            println!("Error: Could not parse '{}'. {}.", url, e);
            "url_filename_parse_error".to_string()
        }
    };

    // path = /12345/video/description+for+this+video
    let vec: Vec<&str> = path.as_str().clone().split("/").map(|s| s).collect();
    let id_5char = vec[1]; // = 12345
    let description = vec[3]; // use for base filename, = description+for+this+video

    let mut base_filename = format!("{}-{}", description, id_5char); // concat then converts &str to String

    if description.len() > MAX_FILENAME_LENGTH {
        let (shorten, _) = description.split_at(MAX_FILENAME_LENGTH); // shorten description
        base_filename = format!("{}...-{}", shorten, id_5char); // concat using shorten description
    }

    return windows_filename(linux_filename(base_filename));
}

// replace invalid linux chars with _ underscore
fn linux_filename(in_filename: String) -> String {
    let out_filename = format!(
        "{}",
        in_filename.trim().replace(
            &['|', '\'', '\"', ':', '\'', '\\', '/'][..], // '"', also works for quote char
            r#"_"#
        )
    );

    return out_filename;
}

// replace invalid windows chars with _ underscore
fn windows_filename(in_filename: String) -> String {
    let out_filename = format!(
        "{}",
        in_filename
            .trim()
            // also replace newline char
            // replacing plus '+' char is specific to spankbang
            .replace(&['<', '>', ':', '?', '*', '\n', '+'][..], r#"_"#) // replace with underscore char
    );

    return out_filename;
}
