extern crate threadpool;
extern crate glob;

use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::env;

use threadpool::ThreadPool;
use glob::Pattern;

enum VisitResult {
    NewPath(PathBuf),
    Done
}

fn visit_dirs(dir: &Path, tx: Sender<VisitResult>, pattern: &Pattern) -> io::Result<()> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path  = entry.path();
            let path_for_name = entry.path();

            let is_match = path_for_name.file_name().and_then(|p| { p.to_str() })
                                                    .and_then(|file_name| { Some(pattern.matches(file_name)) })
                                                    .unwrap_or(false);

            if is_match {
                println!("{}", path_for_name.to_str().unwrap_or("Error: Failed to get name for matching file"));
            }

            if try!(fs::metadata(&path)).is_dir() {
                match tx.send(VisitResult::NewPath(path)) {
                    Ok(_)  => (),
                    Err(_) => println!("Failed to recurse directory from thread"),
                }
            }
        }
    }
    tx.send(VisitResult::Done).unwrap();
    Ok(())
}

fn find(needle: &str) {
    let pattern = Arc::new(Pattern::new(needle).ok().expect("Not a valid glob pattern"));
    let pool = ThreadPool::new(8);
    let (tx, rx) = channel();

    visit_dirs(Path::new("."), tx.clone(), &pattern.clone()).ok().expect("Failed to start search in current directory");

    // remaining `Done` results to be received from worker threads
    let mut remaining = 1;

    while remaining > 0 {
        let res = rx.recv().unwrap();
        match res {
            VisitResult::NewPath(path) => {
                remaining += 1;
                let tx_clone      = tx.clone();
                let tx_clone2     = tx.clone(); // horrible, horrible...
                let pattern_clone = pattern.clone();

                pool.execute(move || {
                    match visit_dirs(&path, tx_clone, &pattern_clone) {
                        Ok(_) => (),
                        Err(e) => {
                            tx_clone2.send(VisitResult::Done).unwrap();
                            println!("Failed to search subdirectory {}: {}", &path.to_str().unwrap(), e);
                        }
                    }
                });
            },

            VisitResult::Done =>
                remaining -= 1
        }
    }
}

fn main() {
    let mut args = env::args();

    args.next().unwrap(); // eat first argument. probably a nicer way to do this

    let arg = args.next();
    match arg {
        Some(needle) => find(&needle),
        None         => println!("Specify file name to look for as an argument")
    }
}

