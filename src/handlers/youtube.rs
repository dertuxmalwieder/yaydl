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
// - YouTube handler -

use crate::definitions::SiteDefinition;

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};

static mut VIDEO_INFO: String = String::new();
static mut VIDEO_MIME: String = String::new();

unsafe fn get_video_info(id: &str) -> Result<Value> {
    if VIDEO_INFO.is_empty() {
        // We need to fetch the video information first.
        let video_url = "https://youtubei.googleapis.com/youtubei/v1/player?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
        let req = ureq::post(&video_url).send_json(ureq::json!({
            "videoId": id,
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": "16.02"
                }
            }
        }))?;
        let body = req.into_string()?;
        VIDEO_INFO = body;
    }

    // Return it:
    let v: Value = serde_json::from_str(&VIDEO_INFO)?;
    Ok(v)
}

// Implement the site definition:
struct YouTubeHandler;
impl SiteDefinition for YouTubeHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        Regex::new(r"(?:www\.)?youtu(?:be.com|be)/")
            .unwrap()
            .is_match(url)
    }

    fn find_video_title<'a>(&'a self, url: &'a str) -> Result<String> {
        let id_regex = Regex::new(r"(?:v=|.be/)(.*$)").unwrap();
        let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();
        unsafe {
            let video_info = get_video_info(id)?;
            let video_info_title = video_info["videoDetails"]["title"].as_str().unwrap_or("");

            Ok(String::from(video_info_title))
        }
    }

    fn find_video_direct_url<'a>(&'a self, url: &'a str, onlyaudio: bool) -> Result<String> {
        let id_regex = Regex::new(r"(?:v=|.be/)(.*$)").unwrap();
        let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();
        unsafe {
            let video_info = get_video_info(id)?;
            let video_info_itags = match video_info["streamingData"]["formats"].as_array() {
                None => return Ok("".to_string()),
                Some(itags) => itags,
            };
            let video_info_itags_adaptive =
                match video_info["streamingData"]["adaptiveFormats"].as_array() {
                    None => return Ok("".to_string()),
                    Some(itags) => itags,
                };

            let mut url_to_choose = "";

            // Finding the least horrible combination of video and audio:
            let vq1 = "tiny";
            let vq2 = "small";
            let vq3 = "medium";
            let vq4 = "large";
            let vq5 = "hd720";
            let vq6 = "hd1080";
            let mut last_vq = "".to_string();

            let aq1 = "AUDIO_QUALITY_LOW";
            let aq2 = "AUDIO_QUALITY_MEDIUM";
            let aq3 = "AUDIO_QUALITY_HIGH";
            let mut last_aq = "".to_string();

            for itag in video_info_itags.iter().chain(video_info_itags_adaptive) {
                // The highest quality wins.
                let this_aq = itag["audioQuality"].as_str().unwrap_or("");
                let this_vq = itag["quality"].as_str().unwrap_or("");

                let is_better_audio = (last_aq.is_empty() && !this_aq.is_empty())
                    || (last_aq == aq1 && (this_aq == aq2 || this_aq == aq3))
                    || (last_aq == aq2 && this_aq == aq3);
                let is_same_or_better_audio = (last_aq == this_aq) || is_better_audio;

                let is_better_video = (last_vq.is_empty() && !this_vq.is_empty())
                    || (last_vq == vq1
                        && (this_vq == vq2
                            || this_vq == vq3
                            || this_vq == vq4
                            || this_vq == vq5
                            || this_vq == vq6))
                    || (last_vq == vq2
                        && (this_vq == vq3 || this_vq == vq4 || this_vq == vq5 || this_vq == vq6))
                    || (last_vq == vq3 && (this_vq == vq4 || this_vq == vq5 || this_vq == vq6))
                    || (last_vq == vq4 && (this_vq == vq5 || this_vq == vq6))
                    || (last_vq == vq5 && this_vq == vq6);
                let is_same_or_better_video = (last_vq == this_vq) || is_better_video;

                let is_better_quality = (is_better_audio && is_same_or_better_video)
                    || (is_better_video && is_same_or_better_audio)
                    || (onlyaudio && is_better_audio);

                // If audio: Try to download the best audio quality.
                // If video: Try to download the best combination.
                if (onlyaudio && itag["mimeType"].to_string().contains("audio/")
                    || !onlyaudio && itag["mimeType"].to_string().contains("video/"))
                    && (!onlyaudio || itag["quality"] != json!(null))
                    && itag["audioQuality"] != json!(null)
                    && (onlyaudio && this_vq.is_empty()
                        || !onlyaudio && last_vq.is_empty() && !this_vq.is_empty())
                    || is_better_quality
                {
                    VIDEO_MIME = itag["mimeType"].to_string();
                    url_to_choose = itag["url"].as_str().unwrap();

                    last_vq = String::from(this_vq);
                    last_aq = String::from(this_aq);
                }
            }

            if url_to_choose.is_empty() {
                Err(anyhow::Error::msg(
                    "Could not find a working itag - aborting.".to_string(),
                ))
            } else {
                Ok(url_to_choose.to_string())
            }
        }
    }

    fn does_video_exist<'a>(&'a self, url: &'a str) -> Result<bool> {
        let id_regex = Regex::new(r"(?:v=|.be/)(.*$)").unwrap();
        let id = id_regex.captures(url).unwrap().get(1).unwrap().as_str();
        unsafe {
            let video_info = get_video_info(id)?;
            let video_info_is_playable = video_info["playabilityStatus"]["status"] == json!("OK");
            let video_info_has_details = video_info["videoDetails"] != json!(null);
            Ok(video_info_has_details && video_info_is_playable)
        }
    }

    fn display_name<'a>(&'a self) -> String {
        "YouTube".to_string()
    }

    fn find_video_file_extension<'a>(&'a self, _url: &'a str, _onlyaudio: bool) -> Result<String> {
        // By this point, we have already filled VIDEO_MIME. Let's just use that.
        unsafe {
            let mut ext = "mp4";
            if VIDEO_MIME.contains("/webm") {
                ext = "webm";
            } else if VIDEO_MIME.contains("audio/mp4") {
                ext = "m4a";
            }

            Ok(ext.to_string())
        }
    }
}

// Push the site definition to the list of known handlers:
inventory::submit! {
    &YouTubeHandler as &dyn SiteDefinition
}
