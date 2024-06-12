use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::sync::mpsc;
use rayon::prelude::*;
use obs_client::Capture;
use show_image::{ImageView, ImageInfo, create_window};
use image::Rgba;

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let capture = Arc::new(Mutex::new(Capture::new("Rainbow Six")));
    let window = create_window("Capture", Default::default())?;

    if capture.lock().unwrap().try_launch().is_err() {
        println!("Failed to launch the capture");
    }

    let (tx, rx) = mpsc::channel();
    let mut counter = 0u32;
    let mut timer = Instant::now();

    rayon::spawn(move || {
        loop {
            let capture = Arc::clone(&capture);
            let tx = tx.clone();
            rayon::spawn(move || {
                let mut binding = capture.lock().unwrap();
                let (frame, (width, height)) = binding.capture_frame::<u8>().unwrap();
                let rgb = bgra_to_rgba(&frame);
                tx.send((rgb, width, height)).unwrap();
            });
        }
    });

    loop {
        let (rgb, width, height) = rx.recv().unwrap();
        let image = ImageView::new(ImageInfo::rgb8(width as u32, height as u32), &*rgb);
        window.set_image("WINDOW", image)?;

        counter += 1;
        if timer.elapsed() >= Duration::from_secs(1) {
            window.!set_title(&format!("Capture - FPS: {}", counter))?;
            counter = 0;
            timer = Instant::now();
        }
    }
}

fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(bgra.len() / 4 * 3); // Adjusted capacity for RGB
    let bgra_chunks = bgra.chunks_exact(4);

    for chunk in bgra_chunks {
        rgb.push(chunk[2]); // R
        rgb.push(chunk[1]); // G
        rgb.push(chunk[0]); // B
        // Alpha channel is ignored
    }

    rgb
}

