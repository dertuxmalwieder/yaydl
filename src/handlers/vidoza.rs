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
// - Vidoza handler -

use crate::agent::{AgentBase, YaydlAgent};
use crate::definitions::SiteDefinition;
use crate::VIDEO;

use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Html> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        // Initialize the agent:
        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        let body = agent
            .get(url)
            .call()
            .expect("Could not go to the url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the site source");

        video.info = body;
    }

    // Return it:
    let d = Html::parse_document(&video.info);
    Ok(d)
}

// Implement the site definition:
struct VidozaHandler;
impl SiteDefinition for VidozaHandler {
    fn can_handle_url<'a>(
        &'a self,
        _video: &mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        Ok(Regex::new(r"vid(oza|ezz).net/.+").unwrap().is_match(url))
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // Vidoza does not seem to have playlists?
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let video_info = get_video_info(video, url)?;

        // Currently, there only is one <H1> on Vidoza. Good for us.
        let h1_selector = Selector::parse("h1").unwrap();
        let text = video_info.select(&h1_selector).next();
        let result = match text {
            Some(value) => value.text().collect::<String>(),
            None => "Vidoza".to_string(),
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
        let video_info = get_video_info(video, url)?;

        let url_selector = Selector::parse("source").unwrap();
        let url_elem = video_info.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("src").unwrap();

        Ok(url_contents.to_string())
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
        "Vidoza".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        _url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        Ok("mp4".to_string())
    }

    fn web_driver_required<'a>(&'a self) -> bool {
        false
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &VidozaHandler as &dyn SiteDefinition
}
