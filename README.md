# EPUB 2 AUDIOBOOK (Local)

Simple project to convert EPUB books into text files for conversion to audiobooks.

Files are optimized for use in [audiobookshelf](https://www.audiobookshelf.org/)

## Supported Text to Speech tools
Currently the only TTS system that is tested is [PiperTTS](https://github.com/rhasspy/piper).  As you need to run this program manually yourself you can use any other local systems.

If you're looking for an all in one that will send the text to Azure TTS and OpenAI TTS check out [https://github.com/p0n1/epub_to_audiobook](https://github.com/p0n1/epub_to_audiobook).

## Features
- Extracts all text from book into individual files
- Embeds the Cover image (if it exists) in the MP3 files
- Export book titles chapters and author for use in scripts later

## How Chapter Titles are handled.
Chapter Titles are currently matched with the Table of Contents, if a match does not exist then the internal (to the epub) is used. This works ok, more work here is needed.

## Installation

### Ubuntu (and other Debian based Linux distributions)
1. Grab the latest [release](https://github.com/haydonryan/epub2audiobook/releases)

2. Install the binary
``` bash
sudo dpkg -i ebook2audiobook_0.1.0-1_amd64.deb
```

### Installation from Source.
Requires the rust toolchain (cargo + rustc)
``` bash
git clone https://github.com/haydonryan/epub2audiobook.git
cd epub2audiobook
```

Development is currently done in main, you probably want to check out a specific release.
``` bash
git checkout v0.1.0
```
Build and install the binary.
``` bash
cargo install --path .
```


## Converting Books (Usage)
You will need [PiperTTS](https://github.com/rhasspy/piper) and [FFmpeg](https://www.ffmpeg.org/) installed.

After converting a decent amount of books to audiobooks, I found there are really a few steps / checkpoints.
1. Initial Conversion from EPUB -> Text

   ``` bash
   ebook2audiobook <epub-filename.epub> <output directory>

    ```
2. Find and replace text.

    Piper doesn't handle certain phrases in 'typical' way eg most of us would read 1904 as 19 O 4, but piper reads it as one thousand nine hundred and four.  This step is critical for enjoyment of the book. (bash script)
    ``` bash
    <coming soon>
    ```
    Currently TTS is expensive (cost of cloud or cpu time locally).  Converting a book with a large index, contents, appendix is a complete waste.  It's best to manually delete files you don't wnat to convert at this checkpoint.


3. TXT -> WAV. via Piper TTS
    Set TTS_THREADS to something that maxes out your CPU. On a Ryzen 5950x 2-3 works well (they spawn sub threads)
    ``` bash
    export TTS_THREADS=3
    cd <output-directory>
    mkdir wav
    ls *.txt | xargs --max-procs=$TTS_THREADS -I % sh -c "cat '%' | <path-piper>/piper --length_scale 0.9 --model <model-path-and-file> --output_file 'wav/%.wav'"
    ```

4. WAV -> MP3 via ffmpeg
   Currently Piper TTS only outputs wav files. The hardest part about converting these to mp3 is to inject the Title, Author, Chapter Title, and Cover into the ID4 tags of the MP3.
    ``` bash
    # From the output directory, not WAV - Important
    ../encode_wav_to_mp3.sh

    ```
    This will encode all WAV files in the WAV directory. Files are placed into an MP3 Directory, Simply move them to your audiobookshelf folder and they will appear.

## Roadmap
- Integrate text replacement (in progress)
- Improve title extraction
- Publishing deb to apt repository
- Publishing binary to AUR

## Why does this project exist?
I've spent a bunch of my spare time learning the Rust programming language.

I was looking for a small project to help learn the language and frameworks.  While other awesome EPUB to Audiobook tools exist, it really irks me that of late everything seems to be written in python.

In my opinion python has it's use cases (for scripting, AI experimentation) but the big problem with the language is that it forces the end user to manage dependencies.  I believe that command line tools (unless for good reason) should be written in compiled (safe) languages.  It doesn't have to be rust - that's just what I used.

It's more efficient, it's easier for end users to use your program, and it's safer. /rant
