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
// - main.rs file -

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    io::{self, copy, Read},
    path::{Path, PathBuf},
};
use url::Url;

mod definitions;
mod ffmpeg;
mod handlers;

#[derive(Parser)]
#[clap(version, about = "Yet Another Youtube Down Loader", long_about = None)]
struct Args {
    #[clap(long = "only-audio", short = 'x', help = "Only keeps the audio stream")]
    onlyaudio: bool,

    #[clap(
        long = "keep-temp-file",
        short = 'k',
        help = "Keeps all downloaded data even with --only-audio"
    )]
    keeptempfile: bool,

    #[clap(long, short = 'v', help = "Talks more while the URL is processed")]
    verbose: bool,

    #[clap(
        long = "audio-format",
        short = 'f',
        help = "Sets the target audio format (only if --only-audio is used).\nSpecify the file extension here.",
        default_value = "mp3"
    )]
    audioformat: String,

    #[clap(long = "output", short = 'o', help = "Sets the output file name")]
    outputfile: Option<String>,

    #[clap(long, help = "The port of your web driver (required for some sites)")]
    webdriver: Option<u16>,

    #[clap(help = "Sets the input URL to use", index = 1)]
    url: String,
}

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

fn download(url: &str, filename: &str) -> Result<()> {
    let url = Url::parse(url)?;
    let resp = ureq::get(url.as_str()).call()?;

    // Find the video size:
    let total_size = resp
        .header("Content-Length")
        .unwrap_or("0")
        .parse::<u64>()?;

    let mut request = ureq::get(url.as_str());

    // Display a progress bar:
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
                 .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] {bytes}/{total_bytes} ({eta})")
                 .progress_chars("#>-"));

    let file = Path::new(filename);

    if file.exists() {
        // Continue the file:
        let size = file.metadata()?.len() - 1;
        // Override the range:
        request = ureq::get(url.as_str())
            .set("Range", &format!("bytes={}-", size))
            .to_owned();
        pb.inc(size);
    }

    let resp = request.call()?;
    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: resp.into_reader(),
    };

    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;

    let _ = copy(&mut source, &mut dest)?;

    Ok(())
}

fn main() -> Result<()> {
    // Argument parsing:
    let args = Args::parse();

    let in_url = &args.url;
    inventory::collect!(&'static dyn definitions::SiteDefinition);
    let mut site_def_found = false;

    for handler in inventory::iter::<&dyn definitions::SiteDefinition> {
        // "15:15 And he found a pair of eyes, scanning the directories for files."
        // https://kingjamesprogramming.tumblr.com/post/123368869357/1515-and-he-found-a-pair-of-eyes-scanning-the
        // ------------------------------------

        // Find a known handler for <in_url>:
        if !handler.can_handle_url(in_url) {
            continue;
        }

        if handler.web_driver_required() && args.webdriver.is_none() {
            // This handler would need a web driver, but none is supplied to yaydl.
            println!("{} requires a web driver installed and running as described in the README. Please tell yaydl which port to use (yaydl --webdriver <PORT>) and try again.", handler.display_name());
            continue;
        }

        // This one is it.
        site_def_found = true;
        println!("Fetching from {}.", handler.display_name());

        let mut webdriverport: u16 = 0;
        if args.webdriver.is_some() {
            webdriverport = args.webdriver.unwrap();
        }
        let video_exists = handler.does_video_exist(in_url, webdriverport)?;
        if !video_exists {
            println!("The video could not be found. Invalid link?");
        } else {
            if args.verbose {
                println!("The requested video was found. Processing...");
            }

            let video_title = handler.find_video_title(in_url, webdriverport);
            let vt = match video_title {
                Err(_e) => "".to_string(),
                Ok(title) => title,
            };

            // Usually, we already find errors here.

            if vt.is_empty() {
                println!("The video title could not be extracted. Invalid link?");
            } else {
                if args.verbose {
                    println!("Title: {}", vt);
                }

                let url = handler.find_video_direct_url(in_url, webdriverport, args.onlyaudio)?;
                let ext =
                    handler.find_video_file_extension(in_url, webdriverport, args.onlyaudio)?;

                // Now let's download it:
                let mut targetfile = format!(
                    "{}.{}",
                    vt.trim()
                        .replace(&['|', '\'', '\"', ':', '\'', '\\', '/'][..], r#""#),
                    ext
                );

                if let Some(in_targetfile) = args.outputfile {
                    targetfile = in_targetfile.to_string();
                }

                if args.verbose {
                    println!("Starting the download.");
                }

                download(&url, &targetfile)?;

                // Convert the file if needed.
                let outputext = args.audioformat;
                if args.onlyaudio && ext != outputext {
                    if args.verbose {
                        println!("Post-processing.");
                    }

                    let inpath = Path::new(&targetfile);
                    let mut outpathbuf = PathBuf::from(&targetfile);
                    outpathbuf.set_extension(outputext);
                    let outpath = &outpathbuf.as_path();

                    ffmpeg::to_audio(inpath, outpath);

                    // Get rid of the evidence.
                    if !args.keeptempfile {
                        fs::remove_file(&targetfile)?;
                    }

                    // Success!
                    println!(
                        "\"{}\" successfully downloaded.",
                        outpathbuf
                            .into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| targetfile.to_string())
                    );
                } else {
                    // ... just success!
                    println!("\"{}\" successfully downloaded.", &targetfile);
                }
            }

            // Stop looking for other handlers:
            break;
        }
    }

    if !site_def_found {
        println!(
            "yaydl could not find a site definition that would satisfy {}. Exiting.",
            in_url
        );
    }

    Ok(())
}
