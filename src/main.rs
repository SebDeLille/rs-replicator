mod copy;

use std::path::Path;
use std::env;
use notify::{RecursiveMode, Watcher, RecommendedWatcher, Config};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use notify::EventKind::{Create, Modify, Remove};
use serde::Deserialize;
use crate::copy::{ChangeType, FileChange, manage_change};

#[derive(Deserialize)]
struct ReplicatorConfig {
    source_path: String,
    destination_path: String,
}

fn init_thread(destination: &String) -> Sender<FileChange> {
    let (tx_event, rx_event): (Sender<FileChange>, Receiver<FileChange>) = channel();

    let t = thread::spawn(move || {
        loop {
            let msg = rx_event.recv().unwrap();
            if msg.kind == ChangeType::STOP {
                break;
            }
            manage_change(&msg);
        }
    });
    tx_event
}

fn path_reader(source: &String, destination: &String) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let tx_event = init_thread(destination);

    watcher.watch(Path::new(source), RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    Create(f) => println!("create {:?}",f),
                    Modify(_) => {
                        for path in event.paths {
                            if let Some(pp)= path.to_str() {
                                tx_event.send(FileChange {
                                    kind: ChangeType::CHANGE,
                                    path: String::from(pp),
                                    destination: destination.clone(),
                                });
                            }
                        }
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

fn read_config(filepath: &String) -> Result<ReplicatorConfig, &'static str> {
    let cfg_file = std::fs::read_to_string(filepath);
    match cfg_file {
        Ok(v) => {
            match toml::from_str(v.as_str()) {
                Ok(config) => Ok(config),
                Err(e)  => {
                    println!("{}", e);
                    Err("Lecture du fichier de configuration impossible.")
                },
            }
        },
        Err(_e) => Err("Fichier de configuration introuvable."),
    }
}

fn main() -> notify::Result<()> {

    let args : Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Bad number of arguments.");
        std::process::exit(1);
    }

    let config: ReplicatorConfig;
    match read_config(&args[1]) {
        Ok(v) => config = v,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }

    path_reader(&config.source_path, &config.destination_path)
}
