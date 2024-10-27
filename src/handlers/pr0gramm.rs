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
// - pr0gramm handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::VIDEO;

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Html> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        // For pr0gramm, we'll need the /static/ site:
        let id_regex = Regex::new(r"(?:uploads/)([0-9]+)$").unwrap();
        let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();

        let static_url = format!("https://pr0gramm.com/static/{}", id).to_owned();
        
        let mut agent = ureq::agent();
        let url_p = Url::parse(&static_url)?;

        if let Some(env_proxy) = env_proxy::for_url(&url_p).host_port() {
            // Use a proxy:
            let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
            agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        }

        let req = agent.get(&static_url).call()?;
        let body = req.into_string()?;
        video.info.push_str(body.as_str());
    }

    // Return it:
    let d = Html::parse_document(&video.info);
    Ok(d)
}

// Implement the site definition:
struct Pr0grammHandler;
impl SiteDefinition for Pr0grammHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"pr0gramm.com/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        let video_info = get_video_info(video, url)?;

        let title_selector = Selector::parse("title").unwrap();
        let text = video_info.select(&title_selector).next();
        let result = match text {
            Some(value) => value.text().collect::<String>(),
            None => "pr0gramm".to_string(),
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

        let url_selector = Selector::parse("video").unwrap();
        let url_elem = video_info.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("src").unwrap();

        // we need to prefix "https":
        let url_final = format!("https:{}", url_contents);

        Ok(url_final.to_string())
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
        "pr0gramm".to_string()
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
    &Pr0grammHandler as &dyn SiteDefinition
}
