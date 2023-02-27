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
// - PornDoe handler -

use crate::definitions::SiteDefinition;

use anyhow::{anyhow, Result};
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
            c.goto(&local_url)
                .await
                .expect("could not go to the site URL");

            // Dismiss the age gate:
            c.execute(
                "document.getElementsByClassName('age-btn')[0].click();",
                vec![],
            )
            .await
            .expect("could not dismiss the age gate");

            let body = c.source().await.expect("could not read the site source");
            video.info.push_str(body.as_str());
            c.close_window().await.expect("could not close the window");
        });
    }

    Ok(true)
}

// Implement the site definition:
struct PornDoeHandler;
impl SiteDefinition for PornDoeHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"porndoe.com/.+").unwrap().is_match(url)
    }

    fn is_playlist<'a>(&'a self, _url: &'a str, _webdriver_port: u16) -> Result<bool> {
        // PornDoe has no playlists.
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

        let h1_selector = Selector::parse("h1.-heading").unwrap();
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
        webdriver_port: u16,
        _onlyaudio: bool,
    ) -> Result<String> {
        let _not_used = get_video_info(video, url, webdriver_port)?;
        let video_info_html = Html::parse_document(video.info.as_str());

        let url_selector = Selector::parse(r#"meta[itemprop="contentUrl"]"#).unwrap();
        let url_elem = video_info_html.select(&url_selector).next().unwrap();
        let url_contents = url_elem.value().attr("content").unwrap();

        Ok(url_contents.to_string())
    }

    fn does_video_exist<'a>(
        &'a self,
        video: &'a mut VIDEO,
        url: &'a str,
        webdriver_port: u16,
    ) -> Result<bool> {
        let _not_used = get_video_info(video, url, webdriver_port);
        Ok(!video.info.is_empty())
    }

    fn display_name<'a>(&'a self) -> String {
        "PornDoe".to_string()
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
    &PornDoeHandler as &dyn SiteDefinition
}
