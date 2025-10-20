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
// - download.rs file -

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use nom::Finish;
use std::{
    fs,
    io::{self, copy, Read},
    path::Path,
};
use url::Url;

use crate::agent::{AgentBase, YaydlAgent};

struct DownloadProgress<'a, R> {
    inner: R,
    progress_bar: &'a ProgressBar,
}

impl<R: Read> Read for DownloadProgress<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

pub fn download_from_playlist(url: &str, filename: &str, verbose: bool) -> Result<()> {
    // Download the playlist file into the temporary directory:
    if verbose {
        println!("{}", "Found a playlist. Fetching ...");
    }

    let mut url = Url::parse(url)?;
    let agent = YaydlAgent::init(url.clone());

    let request = agent.get(url.as_str()).header("Referer", url.as_str());
    let playlist_text = request
        .call()
        .expect("Could not go to the playlist url")
        .body_mut()
        .read_to_string()
        .expect("Could not read the playlist source");

    if verbose {
        println!("{}", "Parsing ...");
    }

    // Parse the playlist:
    let playlist = m3u8_rs::parse_media_playlist(&playlist_text.as_bytes())
        .finish()
        .unwrap();

    // Grab and concatenate the segments from the playlist:
    let file = Path::new(&filename);
    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;

    // Display a progress bar:
    let total_cnt = playlist.1.segments.len() as u64;
    let pb = ProgressBar::new(total_cnt);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    for segment in &playlist.1.segments {
        // .m3u8 playlists are usually relative.
        // Take the original path (from the playlist) and replace
        // the playlist itself by the video (e.g):
        //   playlist URL:  https://foo.bar/play/file.m3u8
        //   playlist item: file1.ts
        //   result:        https://foo.bar/play/file1.ts
        url.path_segments_mut().unwrap().pop().push(&segment.uri);

        let mut request = agent
            .get(url.as_str())
            .header("Referer", url.as_str())
            .call()?;
        let mut source = request.body_mut().as_reader();

        // Note: As we opened the file for appending only,
        // file concatenation happens automatically.
        let _ = copy(&mut source, &mut dest)?;

        // Update the progress bar:
        pb.inc(1);
    }

    pb.finish_and_clear();

    Ok(())
}

pub fn download(url: &str, filename: &str) -> Result<()> {
    let url = Url::parse(url)?;
    let agent = YaydlAgent::init(url.clone());

    let mut resp = agent
        .get(url.as_str())
        .header("Referer", url.as_str())
        .call()?;

    // Find the video size:
    let total_size = resp
        .body()
        .content_length()
        .expect("Failed to read the segment size. Aborting.");

    // Display a progress bar:
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] {percent}%",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let file = Path::new(filename);

    if file.exists() {
        // Continue the file:
        let size = file.metadata()?.len() - 1;
        // Override the range:
        resp = agent
            .get(url.as_str())
            .header("Referer", url.as_str())
            .header("Range", &format!("bytes={}-", size))
            .call()?;
        pb.inc(size);
    }

    let mut source = DownloadProgress {
        progress_bar: &pb,
        inner: resp.body_mut().as_reader(),
    };

    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;

    let _ = copy(&mut source, &mut dest)?;

    pb.finish_and_clear();

    Ok(())
}
