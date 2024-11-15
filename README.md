# EPUB 2 AUDIOBOOK

Simple project to convert EPUB books into text files for conversion to audiobooks.

Files are optimized for use in [audiobookshelf](https://www.audiobookshelf.org/)

## Supported Text to Speech tools
Currently the only TTS system that is tested is [PiperTTS](https://github.com/rhasspy/piper).  As you need to run this program manually yourself you can use any other local systems.

## Installation
Initially until I setup CI, you will need to build this from source.

``` bash
git clone https://github.com/haydonryan/epub2audiobook.git
cd epub2audiobook
cargo build

```

## Usage
You will need [PiperTTS](https://github.com/rhasspy/piper) installed.

``` bash
cargo run <epub-filename.epub>

```

Delete any text files you don't want to include.  You might also need to make some subtitutions with text too for the TTS engine to speak more naturally.

``` bash
export TTS_THREADS=3
mkdir wav
ls *.txt | xargs --max-procs=$TTS_THREADS -I % sh -c "cat % | piper --length_scale 0.9 --model <path to voice> --output_file /wav/%.wav"
```

These files can then be converted to MP3 using ffmpeg.



## Why does this project exist?
I've spent a bunch of my spare time learning the Rust programming language.

I was looking for a small project to help learn the language and frameworks.  While other awesome EPUB to Audiobook tools exist, it really irks me that of late everything seems to be written in python.

In my opinion python has it's use cases (for scripting, AI experimentation) but the big problem with the language is that it forces the end user to manage dependencies.  I believe that command line tools (unless for good reason) should be written in compiled (safe) languages.

It's more efficient, it's easier for end users to use your program, and it's safer.
