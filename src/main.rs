use std::time::Duration;

#[derive(serde::Deserialize)]
struct Img {
	img: String,
	#[serde(default)]
	args: Vec<String>,
}
#[derive(serde::Deserialize)]
struct ConfigList {
	images: Vec<Img>,
	duration_secs: u64,
}

fn main() {
	let mut args = std::env::args();
	args.next();

	let ConfigList {
		images,
		duration_secs,
	} = toml::from_str(
		&std::fs::read_to_string(args.next().map(Into::into).unwrap_or_else(|| {
			let mut home_dir = std::env::home_dir().expect("unable to determine home directory");
			home_dir.push(".config/swloop.toml");
			home_dir
		}))
		.unwrap_or_else(|why| panic!("unable to read file: {why}")),
	)
	.unwrap_or_else(|why| panic!("failed to parse config: {why}"));
	let images: Vec<Vec<String>> = images
		.into_iter()
		.map(|Img { img, mut args }| {
			args.insert(0, String::from("img"));
			args.insert(1, img);
			args
		})
		.collect();
	for args in images.iter().cycle() {
		let mut child = std::process::Command::new("swww")
			.args(args)
			.stderr(std::process::Stdio::piped())
			.spawn()
			.unwrap_or_else(|e| panic!("unable to spawn: {e}"));
		std::thread::sleep(Duration::from_secs(duration_secs));
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
	}
}
