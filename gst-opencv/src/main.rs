use std::path::PathBuf;

use clap::Parser;
use opencv::core::Vector;
use opencv::highgui::{destroy_all_windows, imshow, wait_key};
use opencv::prelude::*;
use opencv::video::create_background_subtractor_mog2;
use opencv::videoio::{CAP_GSTREAMER, VideoCapture};

#[derive(Debug, Parser)]
struct Opt {
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt: Opt = Opt::parse();

    let mut cap = VideoCapture::new(CAP_GSTREAMER, CAP_GSTREAMER)?;
    let params = Vector::new();
    cap.open(&format!("filesrc location={} ! decodebin ! videoconvert ! appsink", opt.file.display()), CAP_GSTREAMER, &params)?;
    let object_detector = create_background_subtractor_mog2(500, 16.0, true)?;

    while cap.is_opened() {
        let mut buf = Vec::with_capacity(1024);
        let ok = cap.read(&mut buf)?;
        if !ok {
            break;
        }

        let mask = object_detector.apply(&buf);
        imshow("Dummy", &mask)?;

        wait_key(1)?;
    }

    destroy_all_windows()?;

    Ok(())
}
