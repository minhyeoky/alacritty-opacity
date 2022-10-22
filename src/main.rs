use clap::Parser;
use regex::Regex;

use shellexpand::env;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::thread;

const LOCK_FILENAME: &str = ".alacritty-opacity-lock";

const CONFIG_FILE_LOCATIONS: [&str; 4] = [
    "$XDG_CONFIG_HOME/alacritty/alacritty.yml",
    "$XDG_CONFIG_HOME/alacritty.yml",
    "$HOME/.config/alacritty/alacritty.yml",
    "$HOME/.alacritty.yml",
];

#[derive(clap::ValueEnum, Clone)]
enum CommandType {
    Increase,
    Decrease,
}

#[derive(Parser)]
#[command(author, version, about = None)]
struct Args {
    #[clap(value_enum)]
    command: CommandType,
    value: f32,
}

fn main() {
    let args = Args::parse();

    if args.value > 1.0 || args.value < 0.0 {
        panic!("The value must be between 0.0 and 1.0");
    }

    let lock_fp = temp_dir().join(LOCK_FILENAME);
    let mut lock_try = 0;
    loop {
        if lock_try >= 5 {
            panic!("Failed to create lock file");
        }

        if lock_fp.exists() {
            thread::sleep(std::time::Duration::from_millis(15));

            #[allow(unused_must_use)]
            {
                fs::remove_file(&lock_fp);
            }
        }

        let lock_f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_fp);
        match lock_f {
            Ok(_) => break,
            Err(_) => {}
        }

        lock_try += 1;
    }

    for each in CONFIG_FILE_LOCATIONS {
        let location = env(each);

        match location.ok() {
            Some(location) => {
                if !Path::new(&location.to_string()).exists() {
                    continue;
                }

                adjust(&location.to_string(), &args);
                break;
            }
            None => continue,
        }
    }

    #[allow(unused_must_use)]
    {
        fs::remove_file(&lock_fp);
    }
}

fn adjust(location: &String, args: &Args) {
    let file = File::open(Path::new(&location)).expect("");
    let reader = io::BufReader::new(file);

    let re_section = Regex::new(r"^.+:$").unwrap();
    let re_opacity = Regex::new(r"^[ \t]+(opacity:)").unwrap();
    let re_number = Regex::new(r"[+-]?((\d+\.?\d*)|(\.\d+))").unwrap();
    let mut is_window_section = false;
    let mut temp_lines = String::new();
    for line in reader.lines() {
        let text = line.expect(&format!(
            "Error occurred while reading config file;location={location}"
        ));
        if re_section.is_match(&text) {
            is_window_section = text.trim() == "window:";
        }

        if is_window_section && re_opacity.is_match(&text) {
            let m = re_number
                .find(&text)
                .expect(&format!("Can not parse opacity line;line={text}"));
            let s = m.as_str();
            let mut v: f32 = m
                .as_str()
                .parse()
                .expect(&format!("Invalid opacity value found;value={s}"));

            match args.command {
                CommandType::Increase => v += args.value,
                CommandType::Decrease => v -= args.value,
            }
            if v < 0.0 {
                v = 0.0;
            }

            if v > 1.0 {
                v = 1.0;
            }

            let mut new_v = "".to_string();
            new_v += &text.to_string()[..m.start()];
            new_v += &v.to_string();
            temp_lines += &new_v;
        } else {
            temp_lines += &text;
        }
        temp_lines += "\n"
    }

    let p = temp_dir().join(&format!("alacritty-opacity-{}", std::process::id()));
    let mut temp_file = File::create(&p).expect("Failed to create tempoary file");
    temp_file
        .write(temp_lines.as_bytes())
        .expect("Failed to write to temporary file");
    temp_file
        .flush()
        .expect("Failed to flush to temporary file");
    fs::rename(
        p,
        Path::new(location)
            .read_link()
            .expect("Failed to resolve symbolic link"),
    )
    .expect("Failed to rename temporary file");
}
