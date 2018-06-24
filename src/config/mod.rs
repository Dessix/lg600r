use ::std::env;
extern crate toml;
extern crate xdg;


const CONFIG_NAME: &str = "config.toml";

pub fn find_dotfile() -> Option<::std::path::PathBuf> {
    // Priorities (highest first):
    //XDG style, e.g.: ~/.config/lg600r/config.toml
    //~/.dotfile, e.g.: ~/.lg600r/config.toml
    xdg::BaseDirectories::with_prefix("lg600r").ok()
        .and_then(|basedirs| {
            basedirs.find_config_file(CONFIG_NAME)
                .and_then(|cfg| {
                    if cfg.exists() { Some(cfg) } else { None }
                })
        })
        .or_else(|| {
            env::home_dir()
                .and_then(|home| {
                    let dotpath = home.join(".lg600r").join(CONFIG_NAME);
                    if dotpath.exists() { Some(dotpath) } else { None }
                })
        })
}

#[derive(Debug)]
struct Configuration {
    bindings: Vec<(u32, String)>,
}

fn parse_config_from_toml_string(tomlstr: &String) -> Option<Configuration> {
    // TODO: This is a total mess; figure out toml/serde support
    use toml::Value as Toml;
    let t: Toml = toml::from_str(tomlstr).ok()?;
    let tbl = t.as_table().unwrap();
    let mut cfg = Configuration { bindings: vec![] };
    for (k, v) in tbl {
        match k.as_ref() {
            "bindings" => { // Yes, you could put this into the file multiple times-- but why?
                if let Toml::Table(items) = v {
                    let binditems: Vec<(u32, String)> = items.into_iter()
                        .map(|(x, y)| (
                            x.as_str().parse::<u32>().unwrap(),
                            String::from(y.as_str().unwrap())
                        ))
                        .collect();
                    for bindpair in binditems {
                        cfg.bindings.push(bindpair)
                    }
                } else {
                    assert!(false);
                }
            },
            _ => (), // ignore other tokens
        }
    };
    return Some(cfg);
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

pub fn load_commands_from_dotfile(dotfilepath: &::std::path::Path) -> Option<Vec<(u32, String)>> {
    let contents = load_dotfile_contents(dotfilepath)
        .expect("Failed to read dotfile contents");
    let Configuration { bindings } =
        parse_config_from_toml_string(&contents)
            .expect("Expected to parse toml from dotfile; toml parse failed?");
    Some(bindings)
}

