use std::fs::metadata;
use std::fs::{self, File};
use std::io::{BufReader, Read};

struct Config {
    query: String,
    path: String,
    case_sensitive: bool,
}

impl Config {
    fn new<'a>(args: Vec<String>) -> Result<Config, &'a str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        if args.len() > 4 {
            Err("too many arguments")?
        }

        if let [_, query, path, ..] = args.as_slice() {
            let case_sensitive = if args.len() == 4 {
                if args[3] == "-I" || args[3] == "--ignore-case" || args[3] == "-i" {
                    true
                } else {
                    Err("invalid third argument")?
                }
            } else {
                false
            };

            Ok(Config {
                query: query.to_string(),
                path: path.to_string(),
                case_sensitive,
            })
        } else {
            Err("invalid arguments")
        }
    }

    fn get_files_from_path(&self) -> () {
        let mut queue: Vec<String> = Vec::new();
        let mut match_count = 0;
        queue.push(self.path.clone());
        loop {
            //get the metadata of of the first element in the queue
            let path = queue.iter().collect::<Vec<&String>>()[0].clone();
            let path_clone = path.clone();
            if let Ok(md) = metadata(path) {
                queue.remove(0);
                if md.is_dir() {
                    if let Ok(entries) = fs::read_dir(path_clone) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path().to_str().unwrap().to_string();
                                queue.push(path);
                            }
                        }
                    }
                } else {
                    match_count += self.process_file(&path_clone);
                }
            } else {
                println!("{} is not a valid path", path_clone);
            }
            if queue.is_empty() {
                println!("{} matches found", match_count);
                return;
            }
        }
    }

    fn process_file(&self, path: &str) -> i32 {
        println!("Now searching {}...", path);
        let mut match_count = 0;
        match File::open(path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut contents = String::new();
                if reader.read_to_string(&mut contents).is_ok() {
                    let matches = self.get_matches(&contents);
                    for (line, i) in matches {
                        match_count += 1;
                        println!("{}: {}", i + 1, line);
                    }
                } else {
                    println!("could not read file {}", path);
                }
            }
            Err(e) => {
                println!("Error processing file {}: {}", path, e);
            }
        }
        match_count
    }

    fn get_matches<'a>(&self, contents: &'a str) -> Vec<(&'a str, usize)> {
        let mut matches: Vec<(&str, usize)> = vec![];
        for (i, line) in contents.lines().enumerate() {
            if self.case_sensitive {
                if line.contains(&self.query) {
                    matches.push((line, i));
                }
            } else {
                if line.to_lowercase().contains(&self.query.to_lowercase()) {
                    matches.push((line, i));
                }
            }
        }
        matches
    }
}

fn main() {
    let config = Config::new(std::env::args().collect::<Vec<String>>()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    config.get_files_from_path();
}
