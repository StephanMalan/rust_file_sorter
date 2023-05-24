# Rust file sorter

This command-line utility allows the user to quickly sort a variety of different files.

Current features:
 - Compatible with images, videos and documents
 - Discover dupliactes
 - Rename files to appropriate name
 - Accepts following file types:
    - doc
    - docx
    - pdf
    - ppt
    - pptx
    - xls
    - xlsx
    - jpeg
    - jpg
    - png
    - avi
    - mov
    - mp4
 - Parse datetime metadata from file the following standards:
    - exif
    - riff
    - quicktime

## WIP
Features left to implement:
 - Implement the command line operations
 - Implement interim folder feature (allow user to validate files before actioned)
 - All existing files should be processed
 - Ignore file name numbering (img(1).jpg) when processing files
 - Add logging
 - Add testing
 - Maybe swap to using multistage progress bar instead of multiple individual progress bars
 - Use file sizes instead of file numbers for progress calculation
 - Actions should be split up (move, copy, delete)
 - Look into faster file copy (especially for larger files)
