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
///
/// # Arguments
/// * `output_directory: the name of the base output directory
/// # Returns
/// * Nothing
fn create_directory_structure(output_directory: String) {
    let original_text_directory = output_directory.clone() + "/original-text";
    let html_directory = output_directory.clone() + "/HTML";

    if !Path::new(&output_directory).exists() {
        std::fs::create_dir(output_directory).unwrap();
    }
    if !Path::new(&original_text_directory).exists() {
        std::fs::create_dir(original_text_directory).unwrap();
    }
    if !Path::new(&html_directory).exists() {
        std::fs::create_dir(html_directory).unwrap();
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

    for (i, current_section) in spine.iter().enumerate() {
        let path = doc.resources.get(&current_section.idref).unwrap().0.clone();
        let text = doc.get_resource_by_path(&path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        let chapter_number = i + 1;

        let path_string: String = path.to_string_lossy().into();
        println!(
            "Processing chapter {}/{}: Section Name: {} Path: {}",
            chapter_number, number_of_ids, current_section.idref, path_string,
        );

        // Find matching TOC entries, otherwise push an empty string
        let toc_title = doc
            .toc
            .iter()
            .find(|toc| toc.content.to_string_lossy().contains(&path_string))
            .map(|toc| toc.label.clone())
            .unwrap_or_default();

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
        println!("  - Title from Section Tag: <{}>\n", section_tag_title);
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
fn convert_book(
    doc: &mut EpubDoc<BufReader<File>>,
    titles: Vec<String>,
    output_directory: &str,
    custom_replacement_library: Option<Vec<(String, String)>>,
) {
    let number_of_ids = doc.spine.len();
    let spine = doc.spine.clone();

    for (i, current_section) in spine.iter().enumerate() {
        let path = doc.resources.get(&current_section.idref).unwrap().0.clone();
        let text = doc.get_resource_by_path(&path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        let chapter_number = i + 1;
        let title = &titles[i];

        let title_to_use = if title.len() > 2 {
            title
        } else {
            &current_section.idref
        };

        let filename = if title.len() > 2 {
            format!("{:04}_{}", chapter_number, sanitize_filename(title))
        } else {
            format!(
                "{:04}_{}",
                chapter_number,
                sanitize_filename(&current_section.idref)
            )
        };

        println!(
            "Converting Chapter {:>3}/{}: {:<21} Title Source: TOC    Filename: {}",
            chapter_number, number_of_ids, &current_section.idref, filename
        );

        output_to_file(
            output_directory.to_owned() + "/" + &filename + ".title",
            title_to_use,
        );

        output_to_file(
            output_directory.to_owned() + "/HTML/" + &filename + ".html",
            html,
        );

        // Write the original text un changed into the original-text directory
        let text = extract_text_from_html(html);
        output_to_file(
            output_directory.to_owned() + "/original-text/" + &filename + ".txt",
            &text,
        );

        // Cleanse the original-text using built in changes
        let mut cleansed_text = replace_text::clean_text(&text);
        cleansed_text = replace_text::convert_money_to_words(&cleansed_text);
        cleansed_text = replace_text::convert_speed_from_acronyms_to_full_text(&cleansed_text);

        // Perform Text Custom Replacements
        if let Some(ref library) = custom_replacement_library {
            cleansed_text = custom_replacements::process_user_replacements(&cleansed_text, library);
        }

        // Write the cleansed text to the root output directory
        output_to_file(
            output_directory.to_owned() + "/" + &filename + ".txt",
            &cleansed_text,
        );
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
    if args.len() != 3 {
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

    let custom_replacement_library =
        custom_replacements::load_custom_replacements("custom-replacements.conf");

    if custom_replacement_library.is_some() {
        println!("\nFound custom text replacement library\n");
    }

    convert_book(
        &mut doc,
        titles,
        output_directory,
        custom_replacement_library,
    );

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

#[test]
fn test_loading_chapter_titles() {
    // Note: Alice in wonderland was obtained from Project Guttenberg (out of copywrite material) as a test book
    // Alice in wonderland is an example of:
    // TOC Having good chapter names.
    // section name having bad titles (see below)
    // title tag having the book title
    // Processing chapter 11/15: Section Name: item12 Path: OEBPS/4930335415765774629_11-h-9.htm.xhtml
    // - Title from TOC Tag: <CHAPTER IX. The Mock Turtle’s Story>
    // - Title from Title Tag: <Alice’s Adventures in Wonderland | Project Gutenberg>
    // - Title from Section Tag: <>
    //                            section name
    // Converting Chapter   1/15: coverpage-wrapper     Title Source: TOC    Filename: alice-test/0001_coverpage-wrapper
    // Converting Chapter   2/15: pg-header             Title Source: TOC    Filename: alice-test/0002_Alice_s_Adventures_in_Wonderland
    // Converting Chapter   3/15: item4                 Title Source: TOC    Filename: alice-test/0003_CHAPTER_I._Down_the_Rabbit-Hole
    // Converting Chapter   4/15: item5                 Title Source: TOC    Filename: alice-test/0004_CHAPTER_II._The_Pool_of_Tears
    // Converting Chapter   5/15: item6                 Title Source: TOC    Filename: alice-test/0005_CHAPTER_III._A_Caucus-Race_and_a_Long_Tale
    // Converting Chapter   6/15: item7                 Title Source: TOC    Filename: alice-test/0006_CHAPTER_IV._The_Rabbit_Sends_in_a_Little_Bill
    // Converting Chapter   7/15: item8                 Title Source: TOC    Filename: alice-test/0007_CHAPTER_V._Advice_from_a_Caterpillar
    // Converting Chapter   8/15: item9                 Title Source: TOC    Filename: alice-test/0008_CHAPTER_VI._Pig_and_Pepper
    // Converting Chapter   9/15: item10                Title Source: TOC    Filename: alice-test/0009_CHAPTER_VII._A_Mad_Tea-Party
    // Converting Chapter  10/15: item11                Title Source: TOC    Filename: alice-test/0010_CHAPTER_VIII._The_Queen_s_Croquet-Ground
    // Converting Chapter  11/15: item12                Title Source: TOC    Filename: alice-test/0011_CHAPTER_IX._The_Mock_Turtle_s_Story
    // Converting Chapter  12/15: item13                Title Source: TOC    Filename: alice-test/0012_CHAPTER_X._The_Lobster_Quadrille
    // Converting Chapter  13/15: item14                Title Source: TOC    Filename: alice-test/0013_CHAPTER_XI._Who_Stole_the_Tarts_
    // Converting Chapter  14/15: item15                Title Source: TOC    Filename: alice-test/0014_CHAPTER_XII._Alice_s_Evidence
    // Converting Chapter  15/15: pg-footer             Title Source: TOC    Filename: alice-test/0015_THE_FULL_PROJECT_GUTENBERG_LICENSE
    let fixture = "fixtures/alice_in_wonderland_by_lewis_carroll.epub";
    let doc = EpubDoc::new(fixture);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();
    let titles = get_chapter_titles(&mut doc);

    assert_eq!(titles[4], "CHAPTER III. A Caucus-Race and a Long Tale");
}
