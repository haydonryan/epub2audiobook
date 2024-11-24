#! /usr/bin/env bash

echo "Run this from your output directory, not the base repository directory."

mkdir MP3
cd WAV/ > /dev/null
let i=1

echo "Grabbing book title, author and cover file."
source ../book.sh

echo "Encoding MP3s from WAVs.."
for f in *.wav; do
  title_file=$(echo ${f/.txt.wav/} )
  CHAPTER_TITLE=$(cat "../$title_file.title")
#  echo $CHAPTER_TITLE

  # Strip .txt.wav out of output file
  title=${title//_/ }
  outfile=$(echo ${f/.txt.wav/} )
  echo "$outfile"

  if [ -f ../Cover.jpg ]; then
    ffmpeg -loglevel info -ac 1 -channel_layout mono -y -i "$f" -i "../Cover.jpg" -c:v copy -c:a libmp3lame -id3v2_version 3  -metadata:s:v comment="Cover (front)"  -metadata track=$i -metadata title="$CHAPTER_TITLE" -metadata album_artist="$BOOK_AUTHOR" -metadata artist="$BOOK_AUTHOR" -metadata album="$BOOK_TITLE" -q:a 2 "../MP3/$outfile.mp3" &
  else
    ffmpeg -loglevel info -ac 1 -channel_layout mono -y -i "$f"  -metadata track=$i -metadata title="$CHAPTER_TITLE" -metadata album_artist="$BOOK_AUTHOR" -metadata artist="$BOOK_AUTHOR" -metadata album="$BOOK_TITLE" -q:a 2 "../MP3/$outfile.mp3" &
  fi

  let i=i+1
done
wait
echo "Done."
