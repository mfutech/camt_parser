# CAMT53 parser

## What is it about

a simple parser, takes a CAMT53 file and generate a simple csv file with the list of transaction, used for importing in gnucash.

currentyl tested on BCV.ch CAMT53, nothing else.

## Usage

```text
Usage: camt_parser.exe [OPTIONS] [FILE]...

Arguments:
  [FILE]...  file to be parsed CAMT53 format

Options:
  -o, --output <FILE>  Sets the output file to use [default: output.csv]
  -h, --help           Print help
  -V, --version        Print version
```
