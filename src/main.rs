use std::path::Path;
use notify::{RecursiveMode, Watcher, Result, RecommendedWatcher, Config};
use std::sync::mpsc::channel;

fn main() -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => println!("event: {:?}", event),
            Err(error) => println!("error: {:?}", error),
        }
    }
    Ok(())
}
