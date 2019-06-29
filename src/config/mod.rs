extern crate dirs;
extern crate toml;
extern crate xdg;

use super::xdo;

const CONFIG_NAME: &str = "config.toml";

pub fn find_dotfile() -> Option<::std::path::PathBuf> {
    // Priorities (highest first):
    //XDG style, e.g.: ~/.config/lg600r/config.toml
    //~/.dotfile, e.g.: ~/.lg600r/config.toml
    xdg::BaseDirectories::with_prefix("lg600r")
        .ok()
        .and_then(|basedirs| {
            basedirs.find_config_file(CONFIG_NAME).and_then(|cfg| {
                if cfg.exists() {
                    Some(cfg)
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            dirs::home_dir().and_then(|home| {
                let dotpath = home.join(".lg600r").join(CONFIG_NAME);
                if dotpath.exists() {
                    Some(dotpath)
                } else {
                    None
                }
            })
        })
}

#[derive(Debug)]
pub struct Configuration {
    pub bindings: Vec<(u32, BindingType)>,
    pub scancodes: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingType {
    Command(String),
    EmulateKey(xdo::Key),
    EmulateMouse(u8),
    KeySequence(String),
}

fn parse_binding(gkey: &String, token: &toml::Value) -> (u32, BindingType) {
    let gkey = gkey.as_str().parse::<u32>().unwrap();
    let binding = match token {
        toml::value::Value::String(s) => BindingType::Command(s.clone()),
        toml::value::Value::Table(table) => match table.get("type").unwrap().as_str().unwrap() {
            "mouse" => {
                let button = table.get("button").unwrap().as_integer().unwrap();
                BindingType::EmulateMouse(button as u8)
            }
            "keyboard" => {
                let source_str = table.get("key").unwrap().as_str().unwrap();
                let key: Result<xdo::Key, _> = std::str::FromStr::from_str(source_str);
                let key = match key {
                    Ok(key) => key,
                    Err(e) => {
                        eprintln!("Failed to parse {} as a key; err: {}", source_str, &e);
                        panic!()
                    }
                };
                BindingType::EmulateKey(key)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    (gkey, binding)
}

fn parse_config_from_toml_string(
    tomlstr: &String,
) -> Result<Configuration, Box<dyn (::std::error::Error)>> {
    // TODO: This is a total mess; figure out toml/serde support
    use toml::Value as Toml;
    let t: Toml = toml::from_str(tomlstr)?;
    let tbl = t.as_table().unwrap();
    let mut cfg = Configuration {
        bindings: vec![],
        scancodes: vec![],
    };
    for (k, v) in tbl {
        match k.as_ref() {
            "bindings" => {
                // This could occur in the file multiple times-- but why?
                if let Toml::Table(items) = v {
                    let binditems: Vec<(u32, BindingType)> = items
                        .into_iter()
                        .map(|(x, y)| parse_binding(&x, y))
                        .collect();
                    for bindpair in binditems {
                        cfg.bindings.push(bindpair)
                    }
                } else {
                    assert!(false);
                }
            }
            "scancodes" => {
                if let Toml::Table(items) = v {
                    let scancodepairs: Vec<(u32, u32)> = items
                        .into_iter()
                        .map(|(x, y): (&String, &toml::Value)| {
                            (x.parse::<u32>().unwrap(), y.as_integer().unwrap() as u32)
                        })
                        .collect();
                    for codepair in scancodepairs {
                        cfg.scancodes.push(codepair)
                    }
                } else {
                    assert!(false);
                }
            }
            _ => (), // ignore other tokens
        }
    }
    Ok(cfg)
}

#[test]
fn test_parse_config() -> () {
    println!("Starting test...");
    let input = r#"
        [bindings]
        12 = "tadah"
        09 = "wooo"
    "#;
    let res = parse_config_from_toml_string(&String::from(input)).expect("Must pass");
    // Note that table order is by key rather than file ordering
    assert_eq!(res.bindings[0], (9u32, String::from("wooo")));
    assert_eq!(res.bindings[1], (12u32, String::from("tadah")));
}

fn load_dotfile_contents(dotfilepath: &::std::path::Path) -> ::std::io::Result<String> {
    assert!(dotfilepath.exists());
    use std::io::prelude::*;
    let mut f = ::std::fs::OpenOptions::new().read(true).open(dotfilepath)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn load_configuration_from_dotfile(
    dotfilepath: &::std::path::Path,
) -> Result<Configuration, Box<dyn (::std::error::Error)>> {
    let contents = load_dotfile_contents(dotfilepath)?;
    parse_config_from_toml_string(&contents)
}
