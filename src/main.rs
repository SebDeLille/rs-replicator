use std::path::Path;
use notify::{RecursiveMode, Watcher, Result, RecommendedWatcher, Config};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use notify::EventKind::{Create, Modify, Remove};

fn main() -> Result<()> {
    let (tx, rx) = channel();
    let (tx_event, rx_event): (Sender<String>, Receiver<String>) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let thread = thread::spawn(move || {
        while true {
            let msg = rx_event.recv().unwrap();
            println!("thread ->{:?}", msg);
        }
    });

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    Create(f) => println!("create {:?}",f),
                    Modify(f) => {
                        let s = format!("modify {:?}",event.paths);
                        tx_event.send(s.clone());
                    },
                    Remove(f) => println!("remove {:?}",f),
                    _ => (),
                }
            }
            Err(error) => println!("error: {:?}", error),
        }
    }
    Ok(())
}
