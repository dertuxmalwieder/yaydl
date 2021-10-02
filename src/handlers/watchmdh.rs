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
// - WatchMDH handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

static mut VIDEO_INFO: String = String::new();

unsafe fn get_video_info(url: &str) -> Result<Html> {
    if VIDEO_INFO.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let req = ureq::get(&url).call()?;
        let body = req.into_string()?;

        VIDEO_INFO = body;
    }

    // Return it:
    let d = Html::parse_document(&VIDEO_INFO);
    Ok(d)
}

// Implement the site definition:
struct WatchMDHHandler;
impl SiteDefinition for WatchMDHHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"watchmdh.to/.+").unwrap().is_match(url)
    }

    fn find_video_title<'a>(&'a self, url: &'a str) -> Result<String> {
        unsafe {
            let video_info = get_video_info(url)?;

            let title_selector = Selector::parse(r#"meta[property="og:title"]"#).unwrap();
            let title_elem = video_info.select(&title_selector).next().unwrap();
            let title_contents = title_elem.value().attr("content").unwrap();

            Ok(title_contents.to_string())
        }
    }

    fn find_video_direct_url<'a>(&'a self, url: &'a str, _onlyaudio: bool) -> Result<String> {
        unsafe {
            // Find the best video format and the rnd value:
            let re_rnd = Regex::new(r"rnd: '(\d+)'").unwrap();
            let rnd = re_rnd
                .captures(&VIDEO_INFO)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str();

            let re_vid1 = Regex::new("video_alt_url: 'function/0/(.+?)',").unwrap();
            let re_vid2 = Regex::new("video_url: 'function/0/(.+?)',").unwrap();

            let url_contents;

            if re_vid1.is_match(&VIDEO_INFO) {
                url_contents = re_vid1
                    .captures(&VIDEO_INFO)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();
            } else {
                url_contents = re_vid2
                    .captures(&VIDEO_INFO)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();
            }

            // TODO: WatchMDH has added hash encryption to their URLs. Fix...
            Ok(String::from(url_contents) + "?rnd=" + rnd)
        }
    }

    fn does_video_exist<'a>(&'a self, url: &'a str) -> Result<bool> {
        unsafe {
            let _video_info = get_video_info(url);
            Ok(!VIDEO_INFO.is_empty())
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "WatchMDH".to_string()
    }

    fn find_video_file_extension<'a>(&'a self, _url: &'a str, _onlyaudio: bool) -> Result<String> {
        Ok("mp4".to_string())
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &WatchMDHHandler as &dyn SiteDefinition
}
