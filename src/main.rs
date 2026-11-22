use std::fs;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xcap::Monitor;

fn main() {
    fs::create_dir_all("screenshots").expect("failed to create screenshots directory");

    let monitors = Monitor::all().unwrap();
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary())
        .expect("no primary monitor found");

    println!("Saving screenshots every 2s into ./screenshots (press Ctrl+C to stop)...");

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let filename = format!("screenshots/screenshot_{timestamp}.png");

        let image = monitor.capture_image().unwrap();
        image.save(&filename).unwrap();

        println!("Saved {filename} ({}x{})", image.width(), image.height());

        sleep(Duration::from_secs(2));
    }
}
