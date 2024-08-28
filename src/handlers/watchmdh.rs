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
use fantoccini::ClientBuilder;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::runtime;

use crate::VIDEO;

fn get_video_info(video: &mut VIDEO, url: &str, webdriver_port: u16) -> Result<bool> {
    if video.info.is_empty() {
        // We need to fetch the video information first.
        // It will contain the whole body for now.
        let local_url = url.to_owned();

        let rt = runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        rt.block_on(async move {
            let webdriver_url = format!("http://localhost:{}", webdriver_port);
            let c = ClientBuilder::native()
                .connect(&webdriver_url)
                .await
                .expect("failed to connect to web driver");
            c.goto(&local_url).await.expect("could not go to the URL");
            let body = c.source().await.expect("could not read the site source");
            video.info.push_str(body.as_str());
            c.close_window().await.expect("could not close the window");
        });
    }

    Ok(true)
}

// Implement the site definition:
struct WatchMDHHandler;
impl SiteDefinition for WatchMDHHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"watch(mdh|dirty).to/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // WatchMDH has no playlists.
        Ok(false)
    }

    fn find_video_title<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        webdriver_port: u16,
    ) -> Result<String> {
        let _not_used = get_video_info(video, url, webdriver_port)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        let title_selector = Selector::parse(r#"meta[property="og:title"]"#).unwrap();
        let title_elem = video_info_html.select(&title_selector).next().unwrap();
        let title_contents = title_elem.value().attr("content").unwrap();

        Ok(title_contents.to_string())
    }

    fn find_video_direct_url<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        _webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        let _not_used = get_video_info(video, url, _webdriver_port)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        let url_selector = Selector::parse("video").unwrap();
        let url_elem = video_info_html.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("src").unwrap();

        Ok(url_contents.to_string())
    }

    fn does_video_exist<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        webdriver_port: u16,
    ) -> Result<bool> {
        let _video_info = get_video_info(video, url, webdriver_port);
        Ok(!video.info.is_empty())
    }

    fn display_name<'a>(&'a self) -> String {
        "WatchMDH".to_string()
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
        true
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &WatchMDHHandler as &dyn SiteDefinition
}
