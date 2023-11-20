mod copy;
mod error;

use std::collections::HashMap;
use std::path::{Path};
use std::env;
use notify::{RecursiveMode, Watcher, RecommendedWatcher, Config, Event};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use notify::EventKind::{Create, Modify, Remove};
use serde::Deserialize;
use crate::copy::{ChangeType, FileChange, manage_change};
use crate::error::ReplicatorError;


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

    let _ = thread::spawn(move || {
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

fn select_destination(path: &str, config: &ReplicatorConfig) -> Result<(String, String), ReplicatorError> {
    for conf in config.paths.keys() {
        if path.starts_with(conf) {
            return Ok((conf.clone(), config.paths.get(conf).unwrap().path.clone()));
        }
    }
    Err(ReplicatorError::new("Inconnu".to_string()))
}

fn select_exceptions(path: &str, config: &ReplicatorConfig) -> Result<Vec<String>, ReplicatorError> {
    for conf in config.paths.keys() {
        if path.starts_with(conf) {
            return Ok(config.paths.get(conf).unwrap().exception.clone());
        }
    }
    Err(ReplicatorError::new("Inconnu".to_string()))
}

fn create_filechange(path: &str, config: &ReplicatorConfig, kind: ChangeType) -> Result<FileChange, ReplicatorError> {
    let d = select_destination(path, config)?;
    let e = select_exceptions(path, config)?;
    let p = String::from(path).strip_prefix(&d.0).unwrap().to_string();

    Ok(FileChange {
        kind,
        path: p,
        source: d.0,
        destination: d.1,
        exceptions: e,
    })
}

fn init_watcher(watcher: &mut dyn Watcher, config: &ReplicatorConfig) -> Result<(), ReplicatorError> {
    for key in config.paths.keys() {
        if let Err(e) = watcher.watch(Path::new(key), RecursiveMode::Recursive) {
            return Err(ReplicatorError::new(e.to_string()));
        }
    }
    Ok(())
}

fn send_file_change(event: Event, tx_event: &Sender<FileChange>, config: &ReplicatorConfig, t: ChangeType) {
    event.paths.iter().for_each(|path| {
        if let Some(str_path) = path.to_str() {
            match create_filechange(str_path, config, t.clone()) {
                Ok(change) => {
                    if let Err(e) = tx_event.send(change) {
                        println!("{}", e);
                    }
                }
                Err(e) => println!("{}", e.msg())
            }
        }
    });
}

fn path_reader(config: &ReplicatorConfig) -> Result<(), ReplicatorError> {
    let (tx, rx) = channel();
    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(e) => return Err(ReplicatorError::new(e.to_string())),
    };

    let tx_event = init_thread();

    init_watcher(&mut watcher, config)?;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    Create(_) => send_file_change(event, &tx_event, config, ChangeType::NEW),
                    Modify(_) => send_file_change(event, &tx_event, config, ChangeType::CHANGE),
                    Remove(_) => send_file_change(event, &tx_event, config, ChangeType::DELETE),
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

fn main() {
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

    if let Err(e) = path_reader(&config) {
        println!("{}", e.msg());
    }
}
