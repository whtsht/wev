use std::{
    fs::File,
    io::{self, Read},
};

pub fn html_from_www(url: &str) -> reqwest::Result<String> {
    let response = reqwest::blocking::get(url)?;
    response.text()
}

pub fn html_from_local(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
