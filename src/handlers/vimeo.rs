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
// - Vimeo handler -

use crate::agent::{AgentBase, YaydlAgent};
use crate::definitions::SiteDefinition;
use crate::VIDEO;

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use url::Url;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Value> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // Those are hidden behing a config file defined in the page source code.
        // Search for: window.vimeo.clip_page_config.player = {"config_url":"(.+?)"
        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        let body = agent
            .get(url)
            .call()
            .expect("Could not go to the url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the site source");
        let re =
            Regex::new("window.vimeo.clip_page_config.player = .\"config_url\":\"(?P<URL>.+?)\"")
                .unwrap();
        let search = re.captures(&body).unwrap();

        // While we're grepping the source code: Vimeo also hides
        // the video title here.
        let title_re =
            Regex::new("<meta property=\"og:title\" content=\"(?P<TITLE>.+?)\"").unwrap();
        let title_search = title_re.captures(&body).unwrap();
        let video_title = title_search.name("TITLE").map_or("", |t| t.as_str());
        video.title = video_title.to_string();

        // If yaydl stops here, the URL is invalid.
        // TODO: That should be more obvious to the user.
        let video_info_url = search
            .name("URL")
            .map_or("", |u| u.as_str())
            .replace("\\", "");

        // The "config_url" body is a JSON structure.
        // Grab and store it:
        let config_body = agent
            .get(&video_info_url)
            .call()
            .expect("Could not go to the url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the site source");
        video.info.push_str(config_body.as_str());
    }

    // Return it:
    let v: Value = serde_json::from_str(&video.info)?;
    Ok(v)
}

// Implement the site definition:
struct VimeoHandler;
impl SiteDefinition for VimeoHandler {
    fn can_handle_url<'a>(
        &'a self,
        _video: &mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        Ok(Regex::new(r"(?:www\.)?vimeo.com/.+").unwrap().is_match(url))
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // Vimeo seems to have no playlists?
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &mut VIDEO,
        _url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let ret = &video.title;
        Ok(ret.to_string())
    }

    fn find_video_direct_url<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        let id_regex = Regex::new(r"(?:vimeo.com/)(.*$)").unwrap();
        let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();
        let video_info = get_video_info(video, id)?;
        let video_info_streams_progressive =
            match video_info["request"]["files"]["progressive"].as_array() {
                None => return Ok("".to_string()),
                Some(streams) => streams,
            };

        // Vimeo makes it easy for us, as the size grows with the quality.
        // Thus, we can just take the largest width here.
        let mut url = "";
        let mut width = 0u64;
        for stream in video_info_streams_progressive.iter() {
            let this_width = stream["width"].as_u64().unwrap_or(0);
            if this_width > width {
                width = this_width;
                url = stream["url"].as_str().unwrap();
            }
        }

        Ok(url.to_string())
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
        "Vimeo".to_string()
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
    &VimeoHandler as &dyn SiteDefinition
}
