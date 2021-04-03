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
// - VIVO handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use cienli::ciphers::rot::{Rot, RotType};
use regex::Regex;
use scraper::{Html, Selector};
use urlencoding::decode;

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
struct VivoHandler;
impl SiteDefinition for VivoHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"vivo.sx/.+").unwrap().is_match(url)
    }

    fn find_video_title<'a>(&'a self, url: &'a str) -> Result<String> {
        unsafe {
            let video_info = get_video_info(url)?;

            let title_selector = Selector::parse("div.stream-content").unwrap();
            let title_elem = video_info.select(&title_selector).next().unwrap();
            let title_contents = title_elem.value().attr("data-name").unwrap();

            Ok(title_contents.to_string())
        }
    }

    fn find_video_direct_url<'a>(&'a self, _url: &'a str, _onlyaudio: bool) -> Result<String> {
        // VIVO displays the stream URL only after executing JavaScript.
        // It is buried inside the source code and ROT47-encrypted. Bah... :-)
        unsafe {
            let src_re = Regex::new("source: '(?P<SOURCE>.+?)',").unwrap();
            let src_search = src_re.captures(&VIDEO_INFO).unwrap();
            let video_src = src_search.name("SOURCE").map_or("", |t| t.as_str());

            // URL decoding:
            let url_decoded = match decode(video_src) {
                Ok(u) => u,
                _ => unreachable!(),
            };

            // un-ROT47:
            let unrotated = Rot::new(&url_decoded, RotType::Rot47);
            Ok(unrotated.decipher().to_string())
        }
    }

    fn does_video_exist<'a>(&'a self, url: &'a str) -> Result<bool> {
        unsafe {
            let _video_info = get_video_info(url);
            Ok(!VIDEO_INFO.is_empty())
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "VIVO".to_string()
    }

    fn find_video_file_extension<'a>(&'a self, _url: &'a str, _onlyaudio: bool) -> Result<String> {
        Ok("mp4".to_string())
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &VivoHandler as &dyn SiteDefinition
}
