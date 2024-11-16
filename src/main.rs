use epub::doc::EpubDoc;
use scraper::{Html, Selector};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::str;

fn get_title_from_section_tag(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("section").unwrap();
    let input = document.select(&selector).next();

    if input.is_none() {
        return "".to_string();
    }
    let input = input.unwrap();

    if let input = document.select(&selector).next() {
        match input.unwrap().attr("title") {
            Some(input) => input.to_string(),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    }
}

fn get_title_from_title_tag(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();

    let title: String = document
        .select(&selector)
        .flat_map(|element| element.text().collect::<Vec<_>>())
        .collect::<String>();

    title.clone()
}

/// Return if all the strings ar ethe same (or empty)
///
/// # arguments
/// * `strings` - A Vector of strings to search
fn all_strings_the_same(strings: &Vec<String>) -> bool {
    if strings.is_empty() {
        return true; // Empty vector is considered to have all strings the same
    }

    let first_string = &strings[0];
    for string in strings {
        if string != first_string {
            return false;
        }
    }

    true
}

/// Outputs a string to a filename.
///
/// # Arguments
/// * `filename` - The filename(path) to write out to
/// * `contents` - The text string that will be written to the file
///
/// Paths are supported as long as they already exist
fn output_to_file(filename: String, contents: &str) {
    let mut file = match File::create(filename) {
        Ok(file) => file,
        Err(error) => panic!("Could not create file: {}", error),
    };
    if let Err(error) = file.write_all(contents.as_bytes()) {
        panic!("Error writing contents to file: {}", error);
    }
}

/// Outputs the embedded cover (if exists)
///
/// # Arguments
/// * `directory` - The directory that the cover file will be put in.  It will be named Cover and
///                 have the same extnesion as it was embedded with.
/// * `doc` - The epub object
/// # Returns
/// Nothing as we don't want to fail if there are no embedded covers
fn save_cover(directory: String, doc: &mut EpubDoc<BufReader<File>>) {
    // Get Cover

    let cover_data = doc.get_cover().unwrap();
    //let filename = format!("{}/Cover.png", directory);

    let filename = match cover_data.1.as_ref() {
        "image/jpeg" => format!("{}/Cover.jpg", directory),
        "image/png" => format!("{}/Cover.png", directory),
        _ => format!("{}/Cover.png", directory),
    };

    let f = File::create(filename);
    assert!(f.is_ok());
    let mut f = f.unwrap();
    let _resp = f.write_all(&cover_data.0);

    dbg!(cover_data.1);
}

// Errors for main

#[derive(Debug)]
enum Epub2AudiobookError {
    IncorrectNumberOfArguments,
}

impl fmt::Display for Epub2AudiobookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Epub2AudiobookError::IncorrectNumberOfArguments => {
                write!(f, "Incorrect number of arguments provided.")
            }
        }
    }
}

///
/// Main Function
///

fn main() -> Result<(), Epub2AudiobookError> {
    println!("EPUB to TXT Converter");
    println!("---------------------");

    // Grab Command Line Arguments and print usage if incorrect.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!();
        println!("Usage:");
        println!("epub2audiobook <epub-filename.epub> <output-directory>");
        println!();
        //return Err("Incorrect Usage");
        return Err(Epub2AudiobookError::IncorrectNumberOfArguments);
    }
    let filename = &args[1];
    let output_directory = &args[2];

    // Check if directory already exists if it doesn't then create it.
    if !Path::new(output_directory).exists() {
        std::fs::create_dir(output_directory).unwrap();
    }

    // Load the EPUB
    let doc = EpubDoc::new(filename);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();

    // Grab book metadata
    let title = doc.mdata("title");
    let author = doc.mdata("creator");
    let number_of_ids = doc.spine.len();

    // Save the book cover to the output directory
    save_cover(output_directory.to_string(), &mut doc);

    println!("Title: {}", title.unwrap());
    println!("Author: {}", author.unwrap());
    println!("Number of Sections: {}", number_of_ids);
    println!();

    let spine = doc.spine.clone();
    let mut i = 1;
    for current_section in spine {
        let path = doc.resources.get(&current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        let filename = format!("{}/{:04}_{}.txt", output_directory, i, current_section);

        print!(
            "Converting chapter {}/{}: {} ",
            i, number_of_ids, current_section,
        );

        println!("Filename: {}", filename);

        let document = Html::parse_document(html);
        let selector = Selector::parse("body").unwrap();
        let _text: String = document
            .select(&selector)
            .flat_map(|element| element.text().collect::<Vec<_>>())
            .collect::<String>();

        output_to_file(filename, &_text);

        //println!("--------------------------------------------------");
        //print!("HTML: <{}>", html);
        //println!("--------------------------------------------------");
        //print!("TEXT: <{}>", _text);
        //println!("--------------------------------------------------");

        i += 1;
    }
    //dbg!(doc.resources);
    //dbg!(&doc);
    //dbg!(spine.clone());
    return Ok(());
    let toc = doc.toc;
    //dbg!(&toc);
    for t in toc {
        println!(
            "{}                         | {}",
            t.label,
            t.content.to_str().unwrap()
        );
    }

    Ok(())
}
