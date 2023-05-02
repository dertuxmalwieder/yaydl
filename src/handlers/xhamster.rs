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
use nom::Finish;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::VIDEO;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<bool> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let local_url = url.to_owned();
        let mut agent = ureq::agent();

        let url_p = Url::parse(url)?;
        if let Some(env_proxy) = env_proxy::for_url(&url_p).host_port() {
            // Use a proxy:
            let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
            agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        }

        video.info.push_str(
            agent
                .get(&local_url)
                .call()
                .expect("Could not go to the url")
                .into_string()
                .expect("Could not read the site source")
                .as_str(),
        );
    }

    Ok(true)
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

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let _not_used = get_video_info(video, url)?;
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
        let mut agent = ureq::agent();

        let url_p = Url::parse(url)?;
        if let Some(env_proxy) = env_proxy::for_url(&url_p).host_port() {
            // Use a proxy:
            let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
            agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        }

        // Find the playlist first:
        let url_selector = Selector::parse(r#"link[rel="preload"][as="fetch"]"#).unwrap();
        let url_elem = video_info_html.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("href").unwrap();

        let mut playlist_url = Url::parse(url_contents)?;
        let request = agent.get(playlist_url.as_str());
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
