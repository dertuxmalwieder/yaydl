# yaydl

[![Crates.io](https://img.shields.io/crates/v/yaydl)](https://crates.io/crates/yaydl)

**y**et **a**nother **y**outube *(and more)* **d**own **l**oader

    % yaydl "https://www.youtube.com/watch?v=jNQXAC9IVRw"

## How? What? Why?

    % yaydl --help

# Features

* Can download videos.
* Can optionally keep only the audio part of them.
* Could convert the resulting file to something else (requires `ffmpeg`).
* Comes as a single binary (once compiled) - take it everywhere on your thumbdrive, no Python cruft required.

## Currently supported sites

* YouTube.com
* Vimeo.com
* VOE.sx
* WatchMDH.to
* VIVO.sx
* Vidoza.net

There is an easy way to add more supported sites, see below for details.

## Non-features

The list of features is deliberately kept short:

* No output quality choice. `yaydl` assumes that you have a large hard drive and your internet connection is good enough, or else you would stream, not download.
* No complex filters. This is a downloading tool.
* No image file support. Videos only.

## Missing features (patches are welcome)

* `yaydl` currently ignores video meta data (except the title) unless they are a part of the video file.
* Playlists are not supported yet.

## How to install

### From the source code

Install Rust (e.g. with [rustup](https://rustup.rs)), then:

**using Fossil:**

    % fossil clone https://code.rosaelefanten.org/yaydl
    % cd yaydl
    % cargo build --release

**using Git:**

    % git clone https://github.com/dertuxmalwieder/yaydl
    % cd yaydl
    % cargo build --release

### From Cargo

    % cargo install yaydl

### From your package manager

`pkgsrc` (with `pkg_add`):

    % pkg_add yaydl

`pkgsrc` (with `pkgin`):

    % pkgin install yaydl

Other package managers:

* Nobody has provided any other packages for `yaydl` yet. You can help!

## How to contribute code

1. Read and agree to the [Code of ~~Conduct~~ Merit](CODE_OF_CONDUCT.md).
2. Implicitly agree to the [LICENSE](LICENSE). Nobody reads those. I don't either.
3. Find out if anyone has filed a GitHub Issue or even sent a Pull Request yet. Act accordingly.
4. Send me a patch, either via e-mail (`yaydl at tuxproject dot de`), on the IRC or as a GitHub Pull Request. Note that GitHub only provides a mirror, so you'd double my work if you choose the latter. :-)

If you do that well (and regularly) enough, I'll probably grant you commit access to the upstream Fossil repository.

### Add support for new sites

1. Implement `definitions::SiteDefinition` as `handlers/<YourSite>.rs`.
2. Push the new handler to the inventory: `inventory::submit! {  &YourSiteHandler as &dyn SiteDefinition }`
3. Add the new module to `handlers.rs`.
4. Optionally, add new requirements to `Cargo.toml`.
5. Send me a patch, preferably with an example. (I cannot know all sites.)

#### Minimal example that does nothing

```rust
// handlers/noop.rs

use anyhow::Result;
use crate::definitions::SiteDefinition;

struct NoopExampleHandler;
impl SiteDefinition for NoopExampleHandler {
    fn can_handle_url<'a>(&'a self, url: &'a str) -> bool {
        // Return true here, if <url> can be covered by this handler.
        // Note that yaydl will skip all other handlers then.
        true
    }
    
    fn does_video_exist<'a>(&'a self, url: &'a str) -> Result<bool> {
    	// Return true here, if the video exists.
    	false
    }
    
    fn find_video_title<'a>(&'a self, url: &'a str) -> Result<String> {
        // Return the video title from <url> here.
        Ok("".to_string())
    }
    
    fn find_video_direct_url<'a>(&'a self, url: &'a str, onlyaudio: bool) -> Result<String> {
        // Return the direct download URL of the video (or its audio version) here.
        Ok("".to_string())
    }

    fn find_video_file_extension<'a>(&'a self, url: &'a str, onlyaudio: bool) -> Result<String> {
        // Return the designated file extension of the video (or audio) file here.
        Ok("mp4".to_string())
    }

    fn display_name<'a>(&'a self) -> String {
        // For cosmetics, this is the display name of this handler.
        "NoopExample"
    }
}
   
// Push the site definition to the list of known handlers:
inventory::submit! {
    &NoopExampleHandler as &dyn SiteDefinition
}
```

### Fix some bugs or add new features

1. Do so.
2. Send me a patch.

## Donations

Writing this software and keeping it available is eating some of the time which most people would spend with their friends. Naturally, I absolutely accept financial compensation.

* PayPal: [GebtmireuerGeld](https://paypal.me/gebtmireuergeld)
* Liberapay: [Cthulhux](https://liberapay.com/Cthulhux/donate)

Thank you.

## Contact

* Twitter: [@tux0r](https://twitter.com/tux0r)
* IRC: `irc.oftc.net/yaydl`
* Matrix: @Cthulhux:matrix.org
