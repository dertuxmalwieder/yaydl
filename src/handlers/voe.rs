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
// - VOE handler -

use crate::definitions::SiteDefinition;

use anyhow::{anyhow, Result};
use regex::Regex;
use scraper::{Html, Selector};

use crate::VIDEO;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Html> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let req = ureq::get(&url).call()?;
        let body = req.into_string()?;

        video.info = body;
    }

    // Return it:
    let d = Html::parse_document(&video.info);
    Ok(d)
}

// Implement the site definition:
struct VoeHandler;
impl SiteDefinition for VoeHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"(?:\.)?voe.sx/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // TODO: Does VOE still have playlists?
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let video_info = get_video_info(video, url)?;

        let h1_selector = Selector::parse("h1.mt-1").unwrap();
        let text = video_info.select(&h1_selector).next();

        let result = match text {
            Some(txt) => txt.text().collect(),
            None => return Err(anyhow!("Erroneous video site - maybe embed-only?")),
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
        let _video_info = get_video_info(video, url)?;
        let url_re = Regex::new("sources: ..src: '(?P<URL>.+?)'").unwrap();
        let url_search = url_re.captures(&video.info).unwrap();
        let video_url = url_search.name("URL").map_or("", |u| u.as_str());

        Ok(video_url.to_string())
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
        "Voe".to_string()
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
    &VoeHandler as &dyn SiteDefinition
}
