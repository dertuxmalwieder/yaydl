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
// - single video file handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use regex::Regex;
use std::path::Path;
use url::Url;

use crate::VIDEO;

// Implement the site definition:
struct GenericFileHandler;
impl SiteDefinition for GenericFileHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"\.mp(4|g)$").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // GenericFile has no playlists.
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<String> {
        // Extract the file name from the URL:
        let filename = Path::new(url)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Ok(filename)
    }

    fn find_video_direct_url<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        // This time, this is simple.
        Ok(url.to_string())
    }

    fn does_video_exist<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        let mut agent = ureq::agent();
        let url_p = Url::parse(url)?;

        if let Some(env_proxy) = env_proxy::for_url(&url_p).host_port() {
            // Use a proxy:
            let proxy = ureq::Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1));
            agent = ureq::AgentBuilder::new().proxy(proxy.unwrap()).build();
        }

        match agent.get(url).call() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "(direct)".to_string()
    }

    fn find_video_file_extension<'a>(
        &'a self,
        _video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        // Just take what's in the URL.
        Ok(String::from(url).split(".").last().unwrap().to_string())
    }

    fn web_driver_required<'a>(&'a self) -> bool {
        false
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &GenericFileHandler as &dyn SiteDefinition
}
