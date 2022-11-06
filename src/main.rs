use std::collections::HashSet;
use std::error::Error;
use std::fs::metadata;
use std::fs::{self, File};
use std::io::{BufReader, Read};

struct Config {
    query: String,
    path: String,
    case_sensitive: bool,
    paths: Vec<String>,
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
                paths: Vec::new(),
            })
        } else {
            Err("invalid arguments")
        }
    }

    fn get_files_from_path(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut files: Vec<String> = Vec::new();
        let mut queue: HashSet<String> = HashSet::new();
        queue.insert(self.path.clone());
        loop {
            //get the metadata of of the first element in the queue
            let path = queue.iter().collect::<Vec<&String>>()[0].clone();
            let path_clone = path.clone();
            //remove the first element from the queue
            if let Ok(md) = metadata(path) {
                queue.remove(&path_clone);
                if md.is_dir() {
                    if let Ok(entries) = fs::read_dir(path_clone) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path().to_str().unwrap().to_string();
                                queue.insert(path);
                            }
                        }
                    }
                } else {
                    files.push(path_clone);
                }
            } else {
                println!("{} is not a valid path", path_clone);
            }
            if queue.is_empty() {
                self.paths = files.to_vec();
                return Ok(files);
            }
        }
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

    fn read_file_from_paths(&self) -> Result<i32, Box<dyn Error>> {
        let mut match_count = 0;
        for path in &self.paths {
            println!("Searching in file: {}...", path);
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
                        println!("Could not read file: {}", path);
                    }
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
        }
        Ok(match_count)
    }
}

fn main() {
    let mut config = Config::new(std::env::args().collect::<Vec<String>>()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    //get paths from path
    let paths = config.get_files_from_path().unwrap_or_else(|err| {
        println!("Problem getting files from path: {}", err);
        std::process::exit(1);
    });
    //read files from paths
    let matches = config.read_file_from_paths().unwrap_or_else(|err| {
        let path_or_paths = if paths.len() > 1 { "paths" } else { "path" };
        println!("Problem reading file from {}: {}", path_or_paths, err);
        std::process::exit(1);
    });
    if matches == 0 {
        println!("No matches found");
    } else {
        println!("Found {} matches", matches);
    }
}
