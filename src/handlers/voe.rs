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

use crate::agent::{AgentBase, YaydlAgent};
use crate::definitions::SiteDefinition;
use crate::VIDEO;

use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

fn resolve_js_redirect(url: &str) -> Result<String> {
    // VOE tends to redirect. Find the actual target URL:
    let static_url = url.to_owned();

    let url_p = Url::parse(&static_url).unwrap();
    let agent = YaydlAgent::init(url_p);

    // We need to fail here if anything goes wrong.
    // To avoid conflicts with the generic download handler, we
    // call this method right in the can_handle_url() method.
    // That means, however, that even direct MP4 downloads are
    // (tried to be) read here. As that will force a
    // BodyExceedsLimit error, we'll just return a "nope", so
    // the fallback to other handlers is considered.
    let body = agent.get(url).call()?.body_mut().read_to_string()?;

    let re_redirect = Regex::new(r"window.location.href = '(?P<URL>.*?)'").unwrap();
    if !re_redirect.is_match(&body) {
        // No redirect
        Ok(String::from(url))
    } else {
        // A redirect...
        let captures = re_redirect.captures(body.as_str()).unwrap();
        let returnval = String::from(captures.name("URL").map_or("", |u| u.as_str()));
        Ok(returnval)
    }
}

fn get_video_info(video: &mut VIDEO, url: &str) -> Result<Html> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        let body = agent
            .get(url)
            .call()
            .expect("Could not go to the url")
            .body_mut()
            .read_to_string()
            .expect("Could not read the site source");
        video.info.push_str(&body);
    }

    // Return it:
    let d = Html::parse_document(&video.info);
    Ok(d)
}

// Implement the site definition:
struct VoeHandler;
impl SiteDefinition for VoeHandler {
    fn can_handle_url<'a>(
        &'a self,
        _video: &mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
    ) -> Result<bool> {
        // We need to catch both VOE.sx and whatever redirectors it uses.
        // As main.rs hasn't built the VIDEO struct here yet, we'll parse
        // the resulting website a first time...
        let url_p = Url::parse(url)?;
        let agent = YaydlAgent::init(url_p);

        let redir_url = &resolve_js_redirect(&url)?;
        let body = agent.get(redir_url).call()?.body_mut().read_to_string()?;

        // If the body contains a VOEPlayer, we're in it.
        Ok(Regex::new(r"VOEPlayer").unwrap().is_match(&body))
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        Ok(true)
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

        // If we're here, we already assume that this is VOE.
        // Maybe we even get a title?
        let result = match text {
            Some(txt) => txt.text().collect::<String>(),
            None => "VOE".to_string(), // embedded video, probably.
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
        let url_re = Regex::new(r#"Node", "(?P<URL>[^"]+)"#).unwrap();
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
