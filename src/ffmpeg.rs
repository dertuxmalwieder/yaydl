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
// - ffmpeg.rs file -

use std::path::Path;
use std::process::Command;

// It makes very little sense to link ffmpeg statically with yaydl.
// Just use the system's one (or inform the user if there isn't one).

pub fn to_audio(inputfile: &Path, outputfile: &Path) {
    Command::new("ffmpeg")
        .arg("-i")
        .arg(inputfile)
        .arg("-vn") // Skip the video streams.
        .arg("-loglevel")
        .arg("quiet") // Shut the fuck up.
        .arg(outputfile)
        .output()
        .expect("Please install ffmpeg to convert the file into audio.");
}

pub fn ts_to_mp4(inputfile: &Path, outputfile: &Path) {
    Command::new("ffmpeg")
        .arg("-i")
        .arg(inputfile)
        .arg("-acodec")
        .arg("copy")
        .arg("-vcodec")
        .arg("copy")
        .arg("-loglevel")
        .arg("quiet") // Shut the fuck up.
        .arg(outputfile)
        .output()
        .expect("Please install ffmpeg to convert the file into MP4.");
}
