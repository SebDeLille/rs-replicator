mod copy;

use std::collections::HashMap;
use std::path::{Path};
use std::env;
use notify::{RecursiveMode, Watcher, RecommendedWatcher, Config};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use notify::EventKind::{Create, Modify, Remove};
use serde::Deserialize;
use crate::copy::{ChangeType, FileChange, manage_change};


#[derive(Deserialize, Debug, Clone)]
struct DestinationConfig {
    path: String,
    exception: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ReplicatorConfig {
    paths: HashMap<String, DestinationConfig>,
}

fn init_thread() -> Sender<FileChange> {
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

fn select_destination(path: &str, config: &ReplicatorConfig) -> Result<String, ()> {
    for conf in config.paths.keys() {
        if path.starts_with(conf) {
            return Ok(config.paths.get(conf).unwrap().path.clone());
        }
    }
    Err(())
}

fn select_exceptions(path: &str, config: &ReplicatorConfig) -> Result<Vec<String>, ()> {
    for conf in config.paths.keys() {
        if path.starts_with(conf) {
            return Ok(config.paths.get(conf).unwrap().exception.clone());
        }
    }
    Err(())
}

fn create_filechange(path: &str, config: &ReplicatorConfig) -> FileChange {
    let d = select_destination(path, config).unwrap();
    let e = select_exceptions(path, config).unwrap();

    FileChange {
        kind: ChangeType::CHANGE,
        path: String::from(path),
        destination: d,
        exceptions: e,
    }
}

fn path_reader(config: &ReplicatorConfig) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let tx_event = init_thread();

    for key in config.paths.keys() {
        println!("{}", key);
        watcher.watch(Path::new(key), RecursiveMode::Recursive)?;
    }


    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    Create(f) => println!("create {:?}", f),
                    Modify(_) => {
                        for path in event.paths {
                            if let Some(pp) = path.to_str() {
                                if let Err(e) = tx_event.send(create_filechange(pp, config)) {
                                    println!("{}", e);
                                }
                            }
                        }
                    }
                    Remove(f) => println!("remove {:?}", f),
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
                Err(e) => {
                    println!("{}", e);
                    Err("Lecture du fichier de configuration impossible.")
                }
            }
        }
        Err(_e) => Err("Fichier de configuration introuvable."),
    }
}

fn main() -> notify::Result<()> {
    let args: Vec<String> = env::args().collect();
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

    path_reader(&config)
}
