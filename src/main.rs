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
}

fn sanitize_filename(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_\.\-\/]").unwrap();
    re.replace_all(input, "_").to_string()
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
    println!();
    println!("=========================");
    println!("= EPUB to TXT Converter =");
    println!("=========================");

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
    let number_of_toc = doc.toc.len();

    // Save the book cover to the output directory
    save_cover(output_directory.to_string(), &mut doc);

    println!("Title: {}", title.clone().unwrap());
    println!("Author: {}", author.clone().unwrap());
    println!("Number of Sections: {}", number_of_ids);
    println!("Number of Items in TOC: {}", number_of_toc);

    println!();

    let includes = format!(
        "#!/bin/bash\n \
                            export BOOK_TITLE=\"{}\" \n \
                            export BOOK_AUTHOR=\"{}\" \n \
                            export BOOK_COVER=\"{}\" \n",
        title.unwrap(),
        author.unwrap(),
        "cover"
    );
    // Save the book Title, Author and CoverName to bash script
    output_to_file(output_directory.to_string() + "/book.sh", &includes);
    //
    // Grab metadata from document to help determine titles
    //
    //
    println!("Grabbing all title options for book");
    println!("-----------------------------------");
    println!();
    let mut title_tag_titles: Vec<String> = Vec::new();
    let mut section_tag_titles: Vec<String> = Vec::new();
    let mut toc_titles: Vec<String> = Vec::new();
    let spine = doc.spine.clone();
    let mut i = 1;
    for current_section in spine {
        let path = doc.resources.get(&current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(path.clone()).unwrap();
        let html = str::from_utf8(&text).unwrap();
        //let toc = doc.toc.clone();

        let p: String = path.to_string_lossy().into();
        println!(
            "Processing chapter {}/{}: Section Name: {} Path: {}",
            i, number_of_ids, current_section, p,
        );

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
    println!("-------------------------------------");
    println!();
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

    //dbg!(section_tag_titles);
    //
    //
    //
    //
    //
    //
    // Final loop to output all the files
    println!();
    println!();
    println!("Converting to Chapters");
    println!("----------------------");
    println!();
    let spine = doc.spine.clone();
    let mut i = 1;
    for current_section in spine {
        let path = doc.resources.get(&current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(&path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        let mut filename = format!("{}/{:04}_{}", output_directory, i, current_section);
        filename = sanitize_filename(&filename);

        // Get any matching TOC items based off filename
        let p: String = path.to_string_lossy().into();
        let mut toc_title = "";
        for d in &doc.toc {
            let toc_path: String = d.content.to_string_lossy().into();
            if toc_path.contains(&p) {
                //println!("Match: {} vs {} = {}", toc_path, p, d.label);
                toc_title = &d.label;
            }
        }

        toc_titles.push(toc_title.to_string());
        //println!("  - Title from TOC Tag: <{}>", toc_title);
        if toc_title.len() > 2 {
            filename = format!("{}/{:04}_{}", output_directory, i, toc_title);
            filename = sanitize_filename(&filename);
            output_to_file(filename.clone() + ".title", toc_title);
        } else {
            output_to_file(filename.clone() + ".title", &current_section);
        }

        print!(
            "Converting Chapter {:>3}/{}: {:<21} ",
            i, number_of_ids, current_section,
        );
        print!("Title Source: TOC    ");
        println!("Filename: {}", filename);
        //println!("resource: {}", path.clone().unwrap().to_str());
        //let tag_title = get_title_from_title_tag(html);
        //println!("  - Title from Title Tag: <{}>", tag_title);

        //let section_title = get_title_from_section_tag(html);
        //println!("  - Title from Section Tag: <{}>", section_title);

        let document = Html::parse_document(html);
        let selector = Selector::parse("body").unwrap();
        let _text: String = document
            .select(&selector)
            .flat_map(|element| element.text().collect::<Vec<_>>())
            .collect::<String>();

        output_to_file(filename.clone() + ".txt", &_text);

        output_to_file(filename + ".html", html);

        i += 1;
    }

    println!();
    println!("Done.");
    println!();

    Ok(())
}
