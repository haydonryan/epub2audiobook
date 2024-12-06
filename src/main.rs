use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::str;

mod custom_replacements;
mod replace_text;

fn get_title_from_section_tag(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("section").unwrap();
    let input = document.select(&selector).next();

    if input.is_none() {
        return "".to_string();
    }

    let input = document.select(&selector).next();
    match input.unwrap().attr("title") {
        Some(input) => input.to_string(),
        None => "".to_string(),
    }
}

fn get_title_from_title_tag(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();

    let title: String = document
        .select(&selector)
        .flat_map(|element| element.text().collect::<Vec<_>>())
        .collect::<String>();

    title.to_owned()
}

/// Return if all the strings are the same (or empty)
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

    if doc.get_cover().is_none() {
        return;
    }

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
}

/// Creates the directory structure
///
/// original-text: original txt files before replacement
/// HTML: Original HTML chapter rip
/// MP3
/// WAV
///
/// # Arguments
/// * `output_directory: the name of the base output directory
/// # Returns
/// * Nothing
fn create_directory_structure(output_directory: String) {
    let original_text_directory = output_directory.clone() + "/original-text";
    let html_directory = output_directory.clone() + "/HTML";
    let mp3_directory = output_directory.clone() + "/MP3";
    let wav_directory = output_directory.clone() + "/WAV";

    if !Path::new(&output_directory).exists() {
        std::fs::create_dir(output_directory).unwrap();
    }
    if !Path::new(&original_text_directory).exists() {
        std::fs::create_dir(original_text_directory).unwrap();
    }
    if !Path::new(&html_directory).exists() {
        std::fs::create_dir(html_directory).unwrap();
    }
    if !Path::new(&mp3_directory).exists() {
        std::fs::create_dir(mp3_directory).unwrap();
    }
    if !Path::new(&wav_directory).exists() {
        std::fs::create_dir(wav_directory).unwrap();
    }
}

/// Removes invalid characters from filenames
///
/// # Arguments
/// * `input` - The string slice of the filename to sanitize
/// # Returns
/// String of the sanitized filename
fn sanitize_filename(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_\.\-\/]").unwrap();
    re.replace_all(input, "_").to_string()
}

/// Extracts text stream from html
///
/// # Arguments
/// * `html` - String to convert
/// # Returns
/// String of unfiltered text
fn extract_text_from_html(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("body").unwrap();
    document
        .select(&selector)
        .flat_map(|element| element.text().collect::<Vec<_>>())
        .collect::<String>()
}

/// Builds a list of chapter titles to use as filenames, and metadata
///
/// # Arguments
/// * `doc` - the epub document
/// # Returns
/// Vector of all the chapter titles (note can contain empty strings, if wasn't able to determine a
/// chapter title.
fn get_chapter_titles(doc: &mut EpubDoc<BufReader<File>>) -> Vec<String> {
    let number_of_ids = doc.spine.len();
    let mut title_tag_titles: Vec<String> = Vec::new();
    let mut section_tag_titles: Vec<String> = Vec::new();
    let mut toc_titles: Vec<String> = Vec::new();
    let spine = doc.spine.clone();
    let mut i = 1;

    for current_section in spine {
        let path = doc.resources.get(&current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(path.clone()).unwrap();
        let html = str::from_utf8(&text).unwrap();

        let p: String = path.to_string_lossy().into();
        println!(
            "Processing chapter {}/{}: Section Name: {} Path: {}",
            i, number_of_ids, current_section, p,
        );

        // Find matching TOC entries, otherwise push an empty string
        let mut toc_title = "";
        for d in &doc.toc {
            let toc_path: String = d.content.to_string_lossy().into();
            if toc_path.contains(&p) {
                //println!("Match: {} vs {} = {}", toc_path, p, d.label);
                toc_title = &d.label;
            }
        }

        toc_titles.push(toc_title.to_string());
        println!("  - Title from TOC Tag: <{}>", toc_title);

        let title_tag_title = get_title_from_title_tag(html);
        if title_tag_title.ne("Cover") {
            title_tag_titles.push(title_tag_title.clone());
            println!("  - Title from Title Tag: <{}>", title_tag_title);
        } else {
            println!("  - Title from Title Tag: <{}> - ignoring", title_tag_title);
        }

        let section_tag_title = get_title_from_section_tag(html);
        section_tag_titles.push(section_tag_title.clone());
        println!("  - Title from Section Tag: <{}>", section_tag_title);
        println!();
        i += 1;
    }

    println!("Applying Rules to decide Title Source");
    println!("-------------------------------------\n");

    if all_strings_the_same(&title_tag_titles) {
        // Title tag is the same - don't use.
        println!("Title Tags all the same or all empty, don't use");
    }
    //dbg!(title_tag_titles);

    if all_strings_the_same(&section_tag_titles) {
        // Title tag is the same - don't use.
        println!("Section Tags all the same or all empty, don't use");
    }
    println!("Hardcoded to use TOC Tags for now.");
    //let titles = toc_titles;
    toc_titles
    //dbg!(section_tag_titles);
}

/// Performs the final processing and outputting of files
///
/// # Arguments
/// * `doc` - the epub document
/// * `titles` - all the chapter titles
/// * `output_directory` - directory to write to.
/// # Returns nothing
fn convert_book(doc: &mut EpubDoc<BufReader<File>>, titles: Vec<String>, output_directory: &str) {
    let number_of_ids = doc.spine.len();
    let spine = doc.spine.clone();

    for (i, current_section) in spine.iter().enumerate() {
        let path = doc.resources.get(current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(&path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        let chapter_number = i + 1;
        let title = &titles[i];

        let title_to_use = if title.len() > 2 {
            title
        } else {
            current_section
        };

        let filename = if title.len() > 2 {
            format!(
                "{}/{:04}_{}",
                output_directory,
                chapter_number,
                sanitize_filename(title)
            )
        } else {
            format!(
                "{}/{:04}_{}",
                output_directory,
                chapter_number,
                sanitize_filename(current_section)
            )
        };

        println!(
            "Converting Chapter {:>3}/{}: {:<21} Title Source: TOC    Filename: {}",
            chapter_number, number_of_ids, current_section, filename
        );

        output_to_file(filename.clone() + ".title", title_to_use);

        let text = extract_text_from_html(html);
        output_to_file(filename.clone() + ".txt", &text);

        output_to_file(filename + ".html", html);
    }
}

/// Create a bash script to provide environment variables for later steps
///
/// # Arguments
/// * `output_directory` - directory to write to.
/// * `title` - title of the book
/// * `author` - author of the book
/// # Returns nothing
fn create_bash_environment(output_directory: &str, title: &str, author: &str) {
    let includes = format!(
        "#!/bin/bash\n \
                            export BOOK_TITLE=\"{}\" \n \
                            export BOOK_AUTHOR=\"{}\" \n \
                            export BOOK_COVER=\"{}\" \n",
        title, author, "cover"
    );
    // Save the book Title, Author and CoverName to bash script
    output_to_file(output_directory.to_string() + "/book.sh", &includes);
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

//
// Main Function
//
fn main() -> Result<(), Epub2AudiobookError> {
    println!("\n=========================");
    println!("= EPUB to TXT Converter =");
    println!("=========================");

    // Grab Command Line Arguments and print usage if incorrect.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("\nUsage:");
        println!("epub2audiobook <epub-filename.epub> <output-directory>\n");
        return Err(Epub2AudiobookError::IncorrectNumberOfArguments);
    }
    let filename = &args[1];
    let output_directory = &args[2];

    create_directory_structure(output_directory.to_string());

    // Load the EPUB
    let doc = EpubDoc::new(filename);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();

    // Grab book metadata
    let title = doc.mdata("title");
    let author = doc.mdata("creator");
    let number_of_ids = doc.spine.len();
    let number_of_toc = doc.toc.len();

    println!("Title: {}", title.clone().unwrap());
    println!("Author: {}", author.clone().unwrap());
    println!("Number of Sections: {}", number_of_ids);
    println!("Number of Items in TOC: {}\n", number_of_toc);

    // Save the book cover to the output directory
    save_cover(output_directory.to_string(), &mut doc);

    // Save a file that has title, and author predefined for ffmpeg later on
    create_bash_environment(output_directory, &title.unwrap(), &author.unwrap());

    // Get chapter titles
    println!("Grabbing all title options for book");
    println!("-----------------------------------\n");
    let titles = get_chapter_titles(&mut doc);

    // Perform the epub to txt conversion
    println!("\n\nConverting to Chapters");
    println!("----------------------\n");

    convert_book(&mut doc, titles, output_directory);

    println!("\nDone.\n");

    Ok(())
}

// ************
// TESTS
// ************

#[test]
fn get_title_from_section_tag_handles_empty_string() {
    assert_eq!(get_title_from_section_tag(""), "");
}

#[test]
fn get_title_from_section_tag_returns_title() {
    let html = r#"<html xmlns="http://www.w3.org/2000/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
        <head>
        <title>title_tag</title>
        <link href="../styles/template.css" rel="stylesheet" type="text/css"/>
        <meta content="urn:uuid" name="meta.content"/>
        </head>
        <body epub:type="bodymatter">
        <section epub:type="bodymatter" id="ch1" title="section_tag">"#;
    assert_eq!(get_title_from_section_tag(html), "section_tag");
}

#[test]
fn get_title_from_section_tag_returns_blank_if_section_tag_missing() {
    let html = r#"<html xmlns="http://www.w3.org/2000/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
        <head>
        <title>title_tag</title>
        <link href="../styles/template.css" rel="stylesheet" type="text/css"/>
        <meta content="urn:uuid" name="meta.content"/>
        </head>
        <body epub:type="bodymatter">"#;
    assert_eq!(get_title_from_section_tag(html), "");
}

#[test]
fn get_title_from_title_tag_handles_empty_string() {
    assert_eq!(get_title_from_title_tag(""), "");
}

#[test]
fn get_title_from_title_tag_returns_title() {
    let html = r#"<html xmlns="http://www.w3.org/2000/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
        <head>
        <title>title_tag</title>
        <link href="../styles/template.css" rel="stylesheet" type="text/css"/>
        <meta content="urn:uuid" name="meta.content"/>
        </head>
        <body epub:type="bodymatter">
        <section epub:type="bodymatter" id="ch1" title="section_tag">"#;
    assert_eq!(get_title_from_title_tag(html), "title_tag");
}

#[test]
fn get_title_from_title_tag_returns_blank_if_section_tag_missing() {
    let html = r#"<html xmlns="http://www.w3.org/2000/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
        <head>
        <link href="../styles/template.css" rel="stylesheet" type="text/css"/>
        <meta content="urn:uuid" name="meta.content"/>
        </head>
        <body epub:type="bodymatter">"#;
    assert_eq!(get_title_from_title_tag(html), "");
}

#[test]
fn test_sanitize_filename_empty_string() {
    assert_eq!("", sanitize_filename(""));
}

// Colons are not allowed in smb storage.
#[test]
fn test_sanitize_filename_replace_colon_with_underscore() {
    assert_eq!("/chapter__1", sanitize_filename("/chapter: 1"));
}

// Spaces are not great in some systems.
#[test]
fn test_sanitize_filename_replace_space_with_underscore() {
    assert_eq!("/chapter_1", sanitize_filename("/chapter 1"));
}

#[test]
fn test_all_strings_the_same() {
    let strings: Vec<String> = vec!["one".to_string(), "two".to_string(), "three".to_string()];
    assert!(!all_strings_the_same(&strings));

    let strings: Vec<String> = vec!["one".to_string(), "one".to_string(), "one".to_string()];
    assert!(all_strings_the_same(&strings));
}
