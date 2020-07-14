mod structs;

use std::fs;
use std::io::prelude::*;
use structs::Resume;
use regex::{Regex, Captures};

pub fn extract_resume(path: &str) -> Resume {
    let mut file = fs::File::open(path)
        .expect("Unable to read the JSON file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Unable to read the JSON file");
    let resume: Resume = serde_json::from_str(&content)
        .expect("Invalid JSON format");

    resume
}

pub fn replace_html_vars(html: &str, resume: Resume) -> String {
    let primitive_vars_re = Regex::new(r"\{\{(?:\s?+?)(\w+.+)(?:\s?+?)\}\}").unwrap();
    let array_vars_re = Regex::new(r"\{!(?:\s?+)(.+)(?:\s?+)([\s\S\w\W]*)!\}").unwrap();
    let array_var_primitive_re = Regex::new(r"\{\s*(\w+)\s*\}").unwrap();

    let resume_in_json: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&resume).unwrap()
    ).expect("Cannot parse JSON");

    let primitives_replaced = primitive_vars_re.replace_all(html, |caps: &Captures| {
        let value = json_get(&resume_in_json, caps[1].to_string());

        remove_quotes(&value.to_string()[..])
    });

    let result = array_vars_re.replace_all(&primitives_replaced, |caps: &Captures| {
        let primary_var_name = caps[1].to_string();
        let html = caps[2].to_string();
        let values = json_get(&resume_in_json, primary_var_name);
        let mut replaced_html = String::new();
        for value in values.as_array().unwrap().iter() {
            let replaced = array_var_primitive_re.replace_all(&html, |c: &Captures| {
                let key = c[1].to_string();
                let res = value[key].to_string();
                res
            });
            // println!("replaced: {:?}", replaced);
            replaced_html.push_str(&replaced);
        }

        remove_quotes(&replaced_html.to_string()[..])
    });

    result.to_string()
}

// Helps to get nested value
fn json_get(json: &serde_json::Value, key_str: String) -> serde_json::Value {
    let keys: Vec<String> = key_str.split(".").map(|s| s.to_string()).collect();
    let mut result: &serde_json::Value = json.get(&keys[0]).unwrap();

    for key in &keys {
        if key == &keys[0] {
            continue;
        }
        let value = result.get(key).unwrap();
        if value.is_string() || value.is_object() || value.is_number() {
            result = value;
        } else {
            result = value;
            break;
        }
    }

    result.to_owned()
}

fn remove_quotes(str: &str) -> String {
    str.replace("\"", "")
}
