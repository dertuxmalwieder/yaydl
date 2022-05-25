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
// - definitions.rs file -

use anyhow::Result;

// Define the public interface for site definitions:
pub trait SiteDefinition {
    // true, if this site can handle <url>.
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool;

    // true, if the video exists.
    fn does_video_exist<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<bool>;

    // true, if the URL is a playlist.
    fn is_playlist<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<bool>;

    // returns the title of a video.
    fn find_video_title<'a>(&'a self, url: &'a str, webdriver_port: u16) -> Result<String>;

    // returns the download URL of a video or playlist.
    fn find_video_direct_url<'a>(
        &'a self,
        url: &'a str,
        webdriver_port: u16,
        onlyaudio: bool,
    ) -> Result<String>;

    // returns the file extension of the video (e.g. "mp4").
    fn find_video_file_extension<'a>(
        &'a self,
        url: &'a str,
        webdriver_port: u16,
        onlyaudio: bool,
    ) -> Result<String>;

    // returns the name of the site (e.g. "YouTube").
    fn display_name<'a>(&'a self) -> String;

    // true, if this site needs a web driver.
    fn web_driver_required<'a>(&'a self) -> bool;
}
