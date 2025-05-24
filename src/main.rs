use std::{ffi::OsString, path::Path, time::Duration};

const USAGE_MESSAGE: &str = "Usage: swloop [image_directory] [duration_in_seconds?]";

fn main() {
	let mut args = std::env::args();
	args.next();
	let image_dir = args.next().expect(USAGE_MESSAGE);
	let duration_secs = args
		.next()
		.map(|d| d.parse::<u32>().expect(USAGE_MESSAGE))
		.unwrap_or(600)
		.max(5);

	let images: Vec<[OsString; 2]> = Path::new(&image_dir)
		.read_dir()
		.unwrap_or_else(|why| panic!("error reading image directory: {why}"))
		.filter_map(|entry| {
			if let Ok(entry) = entry {
				if entry.path().is_file() {
					return Some([OsString::from("img"), entry.path().as_os_str().to_owned()]);
				}
			}
			None
		})
		.collect();

    let duration_secs = duration_secs as u64;

	for args in images.iter().cycle() {
		let mut child = std::process::Command::new("swww")
			.args(args)
			.stderr(std::process::Stdio::piped())
			.spawn()
			.unwrap_or_else(|e| panic!("unable to spawn: {e}"));
		let now = std::time::Instant::now();
		let timeout = Duration::from_secs(5);
		while now.elapsed() < timeout {
			match child.try_wait() {
				Ok(None) => {
					std::thread::sleep(Duration::from_millis(100));
				}
				Err(e) => {
					eprintln!("unable to get child status: {e}");
				}
				_ => {
					break;
				}
			}
		}
		if let Err(why) = child.kill() {
			panic!("unable to kill child: {why}");
		}
		if let Err(why) = child.wait() {
			panic!("waiting on child failed, when it should be killed: {why}");
		}
		std::thread::sleep(Duration::from_secs(duration_secs));
	}
}
