use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::fs::metadata;

struct Config {
    query: String,
    path: String,
    case_sensitive: bool,
    paths: Vec<String>,
}

//new function for Config struct
impl Config {
    fn new<'a>(args: Vec<String>) -> Result<Config, &'a str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        let query = args[1].clone();
        let path = args[2].clone();
        let case_sensitive = if args.len() == 4 {
            if args[3] == "-i" {
                true
            } else {
                false
            }
        } else {
            false
        };
        Ok(Config {
            query,
            path,
            case_sensitive,
            paths: Vec::new(),
        })
    }

    fn get_files_from_path(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut files: Vec<String> = Vec::new();
        let mut queue: HashSet<String> = HashSet::new();
        queue.insert(self.path.clone());
        loop {
            if queue.is_empty() {
                self.paths = files.to_vec();
                return Ok(files);
            }
            //get the metadata of of the first element in the queue
            let path = queue.iter().collect::<Vec<&String>>()[0].clone();
            let path_clone = path.clone();
            let md = metadata(path)?;
            //remove the first element from the queue
            queue = queue.iter().skip(1).cloned().collect();
            if md.is_dir() {
                for entry in fs::read_dir(path_clone)? {
                    let entry = entry?;
                    let path = entry.path();
                    queue.insert(path.to_str().unwrap().to_string());
                }
            } else {
                files.push(path_clone);
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

    fn read_file_from_paths(&self) -> Result<(), Box<dyn Error>> {
        for path in &self.paths {
            let contents = fs::read_to_string(path)?;
            let matches = self.get_matches(&contents);
            if !matches.is_empty() {
                println!("{}:", path);
                for (line, i) in matches {
                    println!("File: {}, Line: {}, Content: {}", path, i, line);
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut config = Config::new(std::env::args().collect::<Vec<String>>()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    //get paths from path
    config.get_files_from_path().unwrap_or_else(|err| {
        println!("Problem getting files from path: {}", err);
        std::process::exit(1);
    });
    //read files from paths
    config.read_file_from_paths().unwrap_or_else(|err| {
        println!("Problem reading files from paths: {}", err);
        std::process::exit(1);
    });
}
