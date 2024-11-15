use epub::doc::EpubDoc;
use scraper::{Html, Selector};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::str;

fn get_title(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();

    let title: String = document
        .select(&selector)
        .flat_map(|element| element.text().collect::<Vec<_>>())
        .collect::<String>();

    title.clone()
}

fn output_to_file(filename: String, contents: &str) {
    let mut file = match File::create(filename) {
        Ok(file) => file,
        Err(error) => panic!("Could not create file: {}", error),
    };
    if let Err(error) = file.write_all(contents.as_bytes()) {
        panic!("Error writing contents to file: {}", error);
    }
}

fn output_cover(directory: String, doc: &mut EpubDoc<BufReader<File>>) {
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

fn main() -> Result<(), Epub2AudiobookError> {
    println!("EPUB to TXT Converter");
    println!("---------------------");

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

    // Load the EPUB
    let doc = EpubDoc::new(filename);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();

    // Grab title and author
    let title = doc.mdata("title");
    let author = doc.mdata("creator");

    output_cover(output_directory.to_string(), &mut doc);
    //dbg!(doc.resources);

    let number_of_ids = doc.spine.len();
    let spine = doc.spine.clone();

    println!("Title: {}", title.unwrap());
    println!("Author: {}", author.unwrap());
    println!("Number of Sections: {}", number_of_ids);
    println!();
    //dbg!(spine.clone());
    let mut i = 1;
    for current_section in spine {
        let path = doc.resources.get(&current_section).unwrap().0.clone();
        let text = doc.get_resource_by_path(path).unwrap();
        let html = str::from_utf8(&text).unwrap();
        print!(
            "Converting chapter {}/{}: {} ",
            i,
            number_of_ids,
            current_section,
            //    path.to_str().unwrap().clone(),
            //doc.resources.get(&current_section).unwrap().1
        );

        let filename = format!("{}/{:04}_{}.txt", output_directory, i, current_section);
        //println!("Filename: {}    Title: {}", filename, current_section);
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
    //dbg!(&doc);
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