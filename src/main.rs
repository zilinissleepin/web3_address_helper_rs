use hotwatch::{Event as HotWatchEvent, EventKind, Hotwatch};
use mac_notification_sys::get_bundle_identifier_or_default;
use notify_rust::{set_application, Notification};
use rdev::{listen, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{Arc, Mutex},
    thread,
    thread::sleep,
    time::Duration,
};

mod appl_script;

use clap::Parser;
#[derive(Parser, Debug)]
#[command(author="zilinissleepin", version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to the configuration file. Default is ./config/address.json"
    )]
    config: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct MemoAddress {
    pub address: String,
    pub label: String,
    pub chain: String,
    pub description: String,
}

use once_cell::sync::Lazy;

static ARGS: Lazy<String> = Lazy::new(get_config_path);

static ADDRESS_DICT: Lazy<HashMap<String, MemoAddress>> = Lazy::new(|| {
    let config = get_config_path();
    init_address_dict(&config)
});

fn main() {
    let _ = &*ADDRESS_DICT;
    let _ = &*ARGS;

    let bundle_id = get_bundle_identifier_or_default("Spotlight");
    if set_application(bundle_id.as_str()).is_err() {
        println!("Failed to set application")
    }

    let mut modifiers = HashSet::new();

    let address_dict = Arc::new(Mutex::new(init_address_dict(&get_config_path())));

    let writer_address_dict = Arc::clone(&address_dict);
    let handle1 = thread::spawn(|| {
        println!("watch file");
        watch_and_reload(ARGS.clone(), writer_address_dict);
    });

    let reader_address_dict = Arc::clone(&address_dict);
    let handle2 = thread::spawn(move || {
        println!("listen callback");

        let callback = move |event: Event| {
            // println!("My callback {:?}", event);
            let data = reader_address_dict.lock().unwrap();
            match event.event_type {
                EventType::KeyPress(key) => match key {
                    Key::MetaLeft | Key::MetaRight => {
                        modifiers.insert(Key::MetaLeft);
                    }
                    Key::KeyJ => {
                        if modifiers.contains(&Key::MetaLeft) {
                            match appl_script::get_selected_text() {
                                Ok(text) => {
                                    let show_text = if text.len() >= 18 {
                                        let prefix: String = text.chars().take(10).collect();
                                        let suffix: String = text
                                            .chars()
                                            .rev()
                                            .take(8)
                                            .collect::<String>()
                                            .chars()
                                            .rev()
                                            .collect();
                                        format!("{}...{}", prefix, suffix)
                                    } else {
                                        text.clone()
                                    };
                                    let f = || get_address_label(&text, &data);
                                    if let Ok(note) = f() {
                                        Notification::new()
                                            .summary(show_text.as_str())
                                            .body(note.as_str())
                                            .show()
                                            .unwrap();
                                    } else {
                                        Notification::new()
                                            .summary(show_text.as_str())
                                            .body("Address Not Found")
                                            .show()
                                            .unwrap();
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Error: {}", err);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                EventType::KeyRelease(key) => match key {
                    Key::MetaLeft | Key::MetaRight => {
                        modifiers.remove(&Key::MetaLeft);
                    }
                    _ => {}
                },
                _ => {}
            }
        };

        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });

    handle1.join().expect("Thread 1 panicked");
    handle2.join().expect("Thread 2 panicked");
}

fn watch_and_reload(path: String, data: Arc<Mutex<HashMap<String, MemoAddress>>>) {
    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    hotwatch
        .watch(path.clone(), move |event: HotWatchEvent| {
            if let EventKind::Modify(_) = event.kind {
                let mut data = data.lock().unwrap();
                *data = init_address_dict(&path);
                println!("Config has changed.");
            }
        })
        .expect("failed to watch file!");

    loop {
        sleep(Duration::from_secs(5))
    }
}

fn get_address_label(
    address: &str,
    address_dict: &HashMap<String, MemoAddress>,
) -> Result<String, String> {
    let parsed_address = address
        .replace("0x", "")
        .trim()
        .to_lowercase()
        .replace(' ', "");
    if let Some(memo_addr) = address_dict.get(&parsed_address) {
        println!("{}", get_msg_from_memo(memo_addr));
        Ok(get_msg_from_memo(memo_addr))
    } else {
        println!("{} Not Found", parsed_address);
        Err("Not Found".to_string())
    }
}

fn get_config_path() -> String {
    let args: Args = Args::parse();
    if let Some(path) = args.config {
        path
    } else {
        "./config/address.json".to_string()
    }
}

fn init_address_dict(config_path: &String) -> HashMap<String, MemoAddress> {
    let file_contents =
        fs::read_to_string(config_path).expect("LogRocket: Should have been able to read the file");
    let address_list: Vec<MemoAddress> = serde_json::from_str(&file_contents).expect("serde err");
    println!("{:#?}", address_list);

    let mut address_dict: HashMap<String, MemoAddress> = HashMap::new();
    for memo_addr in address_list.iter() {
        let parsed_address = memo_addr
            .address
            .replace("0x", "")
            .trim()
            .to_lowercase()
            .replace(' ', "");
        address_dict.insert(parsed_address, memo_addr.clone());
    }

    address_dict
}

fn get_msg_from_memo(memo: &MemoAddress) -> String {
    format!("{} in {}\n{}", memo.label, memo.chain, memo.description,)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_f() {
        init_address_dict(&"./config/address.json".to_string());
    }
}
