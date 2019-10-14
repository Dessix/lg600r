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

fn sval_as_uint(val: &serde_value::Value) -> Option<u64> {
    match val {
        serde_value::Value::String(s) => Some(s.parse::<u64>().unwrap()),
        serde_value::Value::I8(i) => Some((*i) as u64),
        serde_value::Value::I16(i) => Some((*i) as u64),
        serde_value::Value::I32(i) => Some((*i) as u64),
        serde_value::Value::I64(i) => Some((*i) as u64),
        v => None,
    }
}

fn parse_binding(gkey: &String, token: &serde_value::Value) -> (u32, BindingType) {
    let gkey = gkey.as_str().parse::<u32>().unwrap();
    use serde_value::Value;
    use std::convert::{TryFrom, TryInto};
    let binding = match token {
        Value::String(s) => BindingType::Command(s.clone()),
        Value::Map(table) => match table.get(&Value::String("type".to_string())).unwrap() {
            Value::String(s) => {
                match s.as_ref() {
                    "mouse" => {
                        let val_at_button = table.get(&Value::String("button".to_string())).unwrap();
                        let btn: u8 =
                            sval_as_uint(val_at_button)
                                .map(|x| x as u8)
                                .expect("Invalid button value");
                        BindingType::EmulateMouse(btn)
                    }
                    "keyboard" => {
                        if let Value::String(source_str) = table.get(&Value::String("key".to_string())).unwrap() {
                            use std::str::FromStr;
                            let key: Result<xdo::Key, _> = FromStr::from_str(&source_str);
                            let key = match key {
                                Ok(key) => key,
                                Err(e) => panic!("Failed to parse {} as a key; err: {}", source_str, &e)
                            };
                            BindingType::EmulateKey(key)
                        } else {
                            panic!("Key was a non-string value")
                        }
                    }
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
        _ => unreachable!(),
    };

    (gkey, binding)
}

fn parse_config_from_toml_string(
    tomlstr: &String,
) -> Result<Configuration, Box<dyn (::std::error::Error)>> {
    #[derive(Debug)]
    struct BindingWrapper(BindingType);
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    impl serde::Serialize for BindingWrapper {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            let BindingWrapper(inner) = self;
            match inner {
                BindingType::Command(cmd) => {
                    //                    Serialize::serialize()
                    unimplemented!()
                }
                _ => unimplemented!(),
            }
        }
    }
    #[derive(Deserialize)]
    struct IntermedConfig {
        bindings: std::collections::BTreeMap<String, serde_value::Value>,
        scancodes: std::collections::btree_map::BTreeMap<serde_value::Value, serde_value::Value>,
    }
    let icfg: IntermedConfig = toml::from_str(tomlstr)?;
    println!("config.bindings: {:#?}", &icfg.bindings);
    let bindings: Vec<(u32, BindingType)> = icfg
        .bindings
        .iter()
        .map(|(key, val)| parse_binding(key, val))
        .collect();

    let scancodes: Vec<(u32, u32)> = icfg
        .scancodes
        .iter()
        .map(|(key, value)| {
            (
                sval_as_uint(key).map(|x| x as u32).expect("Invalid type in scancode key"),
                sval_as_uint(value).map(|x| x as u32).expect("Invalid type in scancode value"),
            )
        })
        .collect();

    Ok(Configuration {
        bindings,
        scancodes,
    })
}

#[test]
fn test_parse_config() -> () {
    println!("Starting test...");
    let input = r#"
        [bindings]
        12 = "tadah"
        09 = "wooo"
        113 = { type = "mouse", button = 9 }
        115 = { type = "keyboard", key = "F1" }

        [scancodes]
        # non-shifted keys
        007 = 8
        008 = 26
        009 = 30
        010 = 31
        011 = 32
        012 = 33
        013 = 34
        014 = 35
        015 = 36
        016 = 37
        017 = 38
        018 = 39
        019 = 45
        020 = 46
        # g-shifted keys
        104 = 80 # G-shift Backward
        105 = 79 # G-shift Forward
        107 = 81
        108 = 82
        109 = 4
        110 = 17
        111 = 12
        112 = 11
        113 = 7
        114 = 28
        115 = 24
        116 = 13
        117 = 10
        118 = 6
        119 = 25
        120 = 19
    "#;
    let res = parse_config_from_toml_string(&String::from(input)).expect("Must pass");
    assert_eq!(
        res.bindings[0],
        (9u32, BindingType::Command("wooo".to_string()))
    );
    assert_eq!(res.bindings[1], (113u32, BindingType::EmulateMouse(9)));
    assert_eq!(
        res.bindings[2],
        (115u32, BindingType::EmulateKey(crate::xdo::Key::F1))
    );
    assert_eq!(
        res.bindings[3],
        (12u32, BindingType::Command("tadah".to_string()))
    );
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
