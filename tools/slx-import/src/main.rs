use clap::Parser;
use serde::Deserialize;
use serde_with::rust::deserialize_ignore_any;
use serde_xml_rs::from_str;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum Mode {
    Scan,
    List,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Item {
    source(String),
    product(String),
    file(String),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}

#[derive(Debug, Deserialize)]
struct DemoItem {
    #[serde(default)]
    #[serde(rename = "$value")]
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "demos")]
struct Examples {
    #[serde(default)]
    #[serde(rename = "demoitem")]
    demos: Vec<DemoItem>,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    root_dir: Option<String>,

    #[arg(short, long, value_enum, default_value_t = Mode::Scan)]
    mode: Mode,
}

fn scan_examples(root_dir: &str) {
    let root2 = root_dir.clone();
    let root2 = Path::new(&root2);
    // traverse subdirs and find all files match "examples.xml"
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(root_dir) {
        if let Ok(entry) = entry {
            if entry.path().is_file() && entry.path().ends_with("examples.xml") {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    for file in files {
        println!("{}", file.display());
        // This should be relative to root
        let root = Path::new(&root2);
        let relative = file.strip_prefix(root).unwrap().parent().unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        let doc: Examples = from_str(&content).unwrap();
        for demo in &doc.demos {
            for item in &demo.items {
                match item {
                    Item::source(source) => {
                        // lets print relative path from root here along with source
                        println!("openExample('{}/{}')", relative.display(), source);
                    }
                    _ => (),
                }
            }
        }
    }
}

fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let path = Path::new(file.name());
        // only print if the file is under "/systems/" directory and ends with ".xml"
        if path.to_str().map_or(false, |s| s.contains("systems"))
            && path.extension() == Some(std::ffi::OsStr::new("xml"))
        {
            println!(
                "Filename: {} ({}bytes)",
                path.display(), // This safely handles non-UTF8 paths
                file.size()
            );
        }
    }

    Ok(())
}

fn list_examples(root_dir: &str) {
    println!("list_examples");
    let root2 = root_dir.clone();
    let root2 = Path::new(&root2);
    // traverse subdirs and find all files match "examples.xml"
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(root_dir) {
        if let Ok(entry) = entry {
            // println!("{}", entry.path().display());
            if entry.path().is_file()
                && entry.path().extension() == Some(std::ffi::OsStr::new("slx"))
            {
                files.push(entry.path().to_path_buf());
                println!("{}", entry.path().display());
            }
        }
    }
    for file in files {
        println!("{}", file.display());
        let file = std::fs::File::open(&file).unwrap();
        list_zip_contents(&file).unwrap();
    }
}

fn main() {
    let args = Args::parse();
    let root_dir = args
        .root_dir
        .unwrap_or(r"C:\Users\kaido\Documents\MATLAB\Examples\R2024b".to_string());

    if args.mode == Mode::Scan {
        scan_examples(&root_dir);
    }
    if args.mode == Mode::List {
        list_examples(&root_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_examples() {
        let src = r#"<?xml version="1.0"?><demos></demos>"#;
        let doc: Examples = from_str(src).unwrap();
        println!("{:?}", doc);
    }
    #[test]
    fn test_parse_demoitem() {
        let src = r#"<?xml version="1.0"?><demos><demoitem></demoitem></demos>"#;
        let doc: Examples = from_str(src).unwrap();
        println!("{:?}", doc);
        assert_eq!(doc.demos.len(), 1);
    }

    #[test]
    fn test_parse_demoitem1() {
        let src = r#"<?xml version="1.0"?><demos>
            <demoitem>
                <source>example</source>
                <extension>m</extension>
                <file>demo.m</file>
                <product>MATLAB</product>
            </demoitem>
        </demos>"#;
        let doc: Examples = from_str(src).unwrap();
        println!("{:?}", doc);
        assert_eq!(doc.demos.len(), 1);
    }

    #[test]
    fn test_parse_demoitem_files() {
        let src = r#"<?xml version="1.0"?><demos>
            <demoitem>
                <source>example</source>
                <file>demo1.m</file>
                <file component="simulink_industrial" timestamp="1720438223">blah.slx</file>
                <file component="simulink_industrial" timestamp="1720438223">blah.slx</file>
                <product>MATLAB</product>
            </demoitem>
        </demos>"#;
        let doc: Examples = from_str(src).unwrap();
        println!("{:?}", doc);
        assert_eq!(doc.demos.len(), 1);
        //assert_eq!(doc.demos[0].files.len(), 3);
    }
    #[test]
    fn test_parse_failing_duplicat() {
        let src = r#"<?xml version="1.0"?>
            <demos>
            <demoitem>
                <file>html/TwoDegreeofFreedomPIDControlForSetpointTrackingExample.html</file>
                <product>matlab</product>
                <file component="simulink_industrial" open="open" timestamp="1721751476">sldemo_pid2dof.slx</file>
            </demoitem>
            </demos>
            "#;
        let doc: Examples = from_str(src).unwrap();
        println!("{:?}", doc);
        assert_eq!(doc.demos.len(), 1);
        assert_eq!(doc.demos[0].items.len(), 3);
        for item in &doc.demos[0].items {
            match item {
                Item::source(source) => println!("source: {}", source),
                Item::file(file) => println!("file: {}", file),
                Item::product(product) => println!("product: {}", product),
                Item::Other => println!("other"),
            }
        }
    }
}
