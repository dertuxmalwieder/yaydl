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

// Yet Another Youtube Down Loader
// - xHamster handler -

use crate::definitions::SiteDefinition;

use anyhow::{anyhow, Result};
use fantoccini::ClientBuilder;
use nom::Finish;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::runtime;
use url::Url;

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
struct XHamsterHandler;
impl SiteDefinition for XHamsterHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"xhamster.com/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // xHamster has playlists.
        Ok(true)
    }

    fn find_video_title<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<String> {
        unsafe {
            let video_info = get_video_info(url, webdriver_port)?;

            let h1_selector = Selector::parse("h1").unwrap();
            let text = video_info.select(&h1_selector).next();

            let result = match text {
                Some(txt) => txt.text().collect(),
                None => return Err(anyhow!("Could not extract the video title.")),
            };

            Ok(result)
        }
    }

    fn find_video_direct_url<'a>(
        &'a self,
        url: &'a str,
        webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        unsafe {
            let video_info = get_video_info(url, webdriver_port)?;

            // Find the playlist first:
            let url_selector = Selector::parse(r#"link[rel="preload"][as="fetch"]"#).unwrap();
            let url_elem = video_info.select(&url_selector).next().unwrap();
            let url_contents = url_elem.value().attr("href").unwrap();

            let mut playlist_url = Url::parse(url_contents)?;
            let request = ureq::get(playlist_url.as_str());
            let playlist_text = request.call()?.into_string()?;

            // Parse the playlist:
            let playlist = m3u8_rs::parse_media_playlist(&playlist_text.as_bytes())
                .finish()
                .unwrap();

            // Grab the last (= best) segment from the media playlist to find the video "playlist"
            // (which contains all segments of the video):
            let video_uri = &playlist.1.segments.last().ok_or("").unwrap().uri;

            // xHamster uses relative URIs in its playlists, so we'll only need to replace
            // the last URL segment:
            playlist_url
                .path_segments_mut()
                .unwrap()
                .pop()
                .push(video_uri);
            Ok(playlist_url.to_string())
        }
    }

    fn does_video_exist<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<bool> {
        unsafe {
            let _video_info = get_video_info(url, webdriver_port);
            Ok(!VIDEO_INFO.is_empty())
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "xHamster".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        _url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        Ok("ts".to_string())
    }

    fn web_driver_required<'a>(&'a self) -> bool {
        true
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &XHamsterHandler as &dyn SiteDefinition
}
