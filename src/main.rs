use std::fs;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xcap::Monitor;

#[allow(non_upper_case_globals)]
pub static is_timelapse_running: AtomicBool = AtomicBool::new(false);
static CAPTURE_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

pub fn start_timelapse(interval_secs: u64) {
    if is_timelapse_running.swap(true, Ordering::SeqCst) {
        println!("Timelapse is already running.");
        return;
    }

    println!("Starting timelapse (interval: {interval_secs}s)...");

    let handle = thread::spawn(move || {
        if let Err(e) = fs::create_dir_all("screenshots") {
            eprintln!("failed to create screenshots directory: {e}");
            return;
        }

        let monitors = match Monitor::all() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("failed to query monitors: {e}");
                return;
            }
        };

        let monitor = match monitors.into_iter().find(|m| m.is_primary()) {
            Some(m) => m,
            None => {
                eprintln!("no primary monitor found");
                return;
            }
        };

        println!("Saving screenshots every {interval_secs}s into ./screenshots...");

        while is_timelapse_running.load(Ordering::SeqCst) {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let filename = format!("screenshots/screenshot_{timestamp}.png");

            match monitor.capture_image() {
                Ok(image) => match image.save(&filename) {
                    Ok(_) => println!("Saved {filename} ({}x{})", image.width(), image.height()),
                    Err(e) => eprintln!("Failed to save {filename}: {e}"),
                },
                Err(e) => eprintln!("Failed to capture image: {e}"),
            }

            // Sleep in 100ms intervals so stop_timelapse responds promptly
            let intervals = interval_secs.max(1) * 10;
            for _ in 0..intervals {
                if !is_timelapse_running.load(Ordering::SeqCst) {
                    break;
                }
                sleep(Duration::from_millis(100));
            }
        }
    });

    if let Ok(mut lock) = CAPTURE_THREAD.lock() {
        *lock = Some(handle);
    }
}

pub fn stop_timelapse() {
    if !is_timelapse_running.swap(false, Ordering::SeqCst) {
        println!("Timelapse is not running.");
        return;
    }

    println!("Stopping timelapse...");

    if let Ok(mut lock) = CAPTURE_THREAD.lock() {
        if let Some(handle) = lock.take() {
            let _ = handle.join();
        }
    }

    println!("Timelapse stopped.");
}

pub fn clear_screenshots() {
    if let Ok(entries) = fs::read_dir("screenshots") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

fn main() {
    clear_screenshots();
    start_timelapse(1);
    sleep(Duration::from_secs(6));
    stop_timelapse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timelapse_lifecycle() {
        assert!(!is_timelapse_running.load(Ordering::SeqCst));
        start_timelapse(2);
        assert!(is_timelapse_running.load(Ordering::SeqCst));
        stop_timelapse();
        assert!(!is_timelapse_running.load(Ordering::SeqCst));
    }
}
