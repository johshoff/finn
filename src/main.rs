extern crate threadpool;

use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::env;

enum VisitResult {
    NewPath(PathBuf),
    Done
}

fn visit_dirs(dir: &Path, tx: Sender<VisitResult>, needle: &str) -> io::Result<()> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path  = entry.path();

            let path_for_name = entry.path(); //< must be its own let statement. won't live long enough with an added .to_str().unwrap()
            let name = path_for_name.to_str().unwrap();

            if name.contains(needle) {
                println!("{}", name);
            }

            if try!(fs::metadata(&path)).is_dir() {
                tx.send(VisitResult::NewPath(path)).unwrap();
            }
        }
    }
    tx.send(VisitResult::Done).unwrap();
    Ok(())
}

fn find(needle: &str) {
    let needle = Arc::new(needle.to_string());
    let pool = ThreadPool::new(8);
    let (tx, rx) = channel();

    visit_dirs(Path::new("."), tx.clone(), &needle.clone()).ok().expect("Failed to start search in current directory");

    // remaining `Done` results to be received from worker threads
    let mut remaining = 1;

    while remaining > 0 {
        let res = rx.recv().unwrap();
        match res {
            VisitResult::NewPath(path) => {
                remaining += 1;
                let tx_clone     = tx.clone();
                let tx_clone2    = tx.clone(); // horrible, horrible...
                let needle_clone = needle.clone();

                pool.execute(move || {
                    match visit_dirs(&path, tx_clone, &needle_clone) {
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

