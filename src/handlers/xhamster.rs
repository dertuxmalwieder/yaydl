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

use crate::agent::{AgentBase, YaydlAgent};
use crate::definitions::SiteDefinition;
use crate::VIDEO;

use anyhow::{anyhow, Result};
use nom::Finish;
use scraper::{Html, Selector};
use url::Url;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<bool> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        let local_url = url.to_owned();
        let body = agent
            .get(&local_url)
            .call()
            .expect("Could not go to the url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the site source");
        video.info.push_str(&body);
    }

    Ok(true)
}

// Implement the site definition:
struct XHamsterHandler;
impl SiteDefinition for XHamsterHandler {
    fn can_handle_url<'a>(
        &'a self,
        video: &mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        let _not_used = get_video_info(video, url)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        // xHamster URLs contain application-name=="xHamster".
        let app_selector = Selector::parse(r#"meta[name="application-name"]"#).unwrap();
        let app_elem = video_info_html.select(&app_selector).next();
        match app_elem {
            Some(elem) => {
                let app_name = elem.value().attr("content").unwrap();

                match app_name == "xHamster" {
                    true => Ok(true),
                    _ => Ok(false),
                }
            }
            None => Ok(false),
        }
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // xHamster has playlists.
        Ok(true)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let _ = get_video_info(video, url)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        let h1_selector = Selector::parse("h1").unwrap();
        let text = video_info_html.select(&h1_selector).next();

        let result = match text {
            Some(txt) => txt.text().collect(),
            None => return Err(anyhow!("Could not extract the video title.")),
        };

        Ok(result)
    }

    fn find_video_direct_url<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        let _not_used = get_video_info(video, url)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        // Find the playlist first:
        let url_selector = Selector::parse(r#"link[rel="preload"][as="fetch"]"#).unwrap();
        let url_elem = video_info_html.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("href").unwrap();

        let mut playlist_url = Url::parse(url_contents)?;
        let request = agent.get(playlist_url.as_str());
        let playlist_text = request
            .call()
            .expect("Could not go to the playlist url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the playlist source");

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

    fn does_video_exist<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        let _video_info = get_video_info(video, url);
        Ok(!video.info.is_empty())
    }

    fn display_name<'a>(&'a self) -> String {
        "xHamster".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        _url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        Ok("ts".to_string())
    }

    fn web_driver_required<'a>(&'a self) -> bool {
        false
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &XHamsterHandler as &dyn SiteDefinition
}
