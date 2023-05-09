# umi-transfer

A tool for transferring Unique Molecular Identifiers (UMIs) provided as separate FastQ file to the header of records in paired FastQ files.

## Background

To increase the accuracy of quantitative DNA sequencing experiments, Unique Molecular Identifiers may be used. UMIs are short sequences used to uniquely tag each molecule in a sample library and facilitate the accurate identification of read duplicates. They must be added during library preparation and prior to sequencing, therefore require appropriate arrangements with your sequencing provider.

Most tools capable of taking UMIs into consideration during an analysis workflow, expect the respective UMI sequence to be embedded into the read's ID. Please consult your tools' manuals regarding the exact specification.

For some some library preparation kits and sequencing adapters, the UMI sequence needs to be read together with the index from the antisense strand and thus will be output as a separate FastQ file during demultiplexing.

This tools can integrate those separate UMIs into the headers in an efficient manner and can also correct divergent read numbers back to the canonical `1` and `2`.

## Installation

### Compile from source

Given that you have [rust installed](https://www.rust-lang.org/tools/install) on your computer, download this repo and run

```shell
cargo build --release
```

That should create an executable `target/release/umi-transfer` that can be placed anywhere in your `$PATH` or be executed directly by specifying its path:

```shell
./target/release/umi-transfer --version
umi-transfer 0.2.0
```

## Usage

>### Performance Note
>
>The decompression and compression used within umi-transfer is single-threaded, so to get the most reads per minute performance, see the [high performance guide](#high-performance-guide)

The tool requires three FastQ files as input. You can manually specify the names and location of the output files with `--out` and `--out2` or the tool will append a `with_UMI` suffix to your input file names as output. It additionally accepts to choose a custom UMI delimiter with `--delim` and to set the flags `-f`, `-c` and `-z`. The latter specifies to compress the output and `-c` is used to ensure `1` and `2` as read numbers in the output. `-f` / `--force` will overwrite existing output files without prompting the user.

```raw
$ umi-transfer external --help
    umi-transfer-external
Integrate UMIs from a separate FastQ file

USAGE:
    umi-transfer external [OPTIONS] --in <R1_IN> --in2 <R2_IN> --umi <RU_IN>

OPTIONS:
    -c, --correct_numbers    Ensure read numbers 1 and 2 in sequence header of output files.

    -d, --delim <DELIM>      Delimiter to use when joining the UMIs to the read name. Defaults to `:`.

    -f, --force <FORCE>      Overwrite existing output files without further warnings or prompts.

    -h, --help               Print help information
        --in <R1_IN>         [REQUIRED] Input file 1 with reads.


        --in2 <R2_IN>        [REQUIRED] Input file 2 with reads.


        --out <R1_OUT>       Path to FastQ output file for R1.


        --out2 <R2_OUT>      Path to FastQ output file for R2.


    -u, --umi <RU_IN>        [REQUIRED] Input file with UMI.

    -z, --gzip               Compress output files with gzip. By default turned off to encourage use
                             of external compression (see Readme).
```

### Example

```shell
umi-transfer external -f --in 'R1.fastq' --in2 'R3.fastq' --umi 'R2.fastq'
```

### High Performance Guide

If you have more than one thread available on your computer and would like to process the read files as quickly as possible we recommend to use unix FIFOs (First In First Out) to handle decompression and compression of the FastQ files.
This can be done as follows, given that you have your input files compressed as `fastq.gz`, first create FIFOs to represent your uncompressed input files:

```shell
mkfifo read1.fastq
mkfifo read2.fastq
mkfifo read3.fastq
```

and then we use `zcat` to decompress our input files and send it to the pipe that the FIFOs represent:

```shell
$ zcat read1.fastq.gz > read1.fastq &
[1] 233387
$ zcat read2.fastq.gz > read2.fastq &
[2] 233388
$ zcat read3.fastq.gz > read3.fastq &
[3] 233389
```

Note the trailing `&` to leave these processes running in the background. We can inspect the directory with `ls`:

```shell
$ ls -lh
total 1.5K
-rw-rw----. 1 alneberg ngi2016004 4.5G Apr 13 12:18 read1.fastq.gz
-rw-rw----. 1 alneberg ngi2016004 1.1G Apr 13 12:18 read2.fastq.gz
-rw-rw----. 1 alneberg ngi2016004 4.5G Apr 13 12:18 read3.fastq.gz
prw-rw-r--. 1 alneberg ngi2016004    0 Apr 13 12:46 read1.fastq
prw-rw-r--. 1 alneberg ngi2016004    0 Apr 13 12:46 read2.fastq
prw-rw-r--. 1 alneberg ngi2016004    0 Apr 13 12:46 read3.fastq
```

We continue to create corresponding FIFOs for the output files (note that the filenames need to match the value given to `--prefix`)

```shell
$ mkfifo output1.fastq
$ mkfifo output2.fastq
$ pigz -p 10 --stdout > output1.fastq.gz < output1.fastq &
[4] 233394
$ pigz -p 10 --stdout > output2.fastq.gz < output2.fastq &
[5] 233395
```

The value `10` is how many threads each of the `pigz` processes is allowed to use.
The optimal value for this depends on several factors and for optimal performance you will have to do some testing on your exact hardware.
We can then run the `umi-transfer` program as follows:

```shell
umi-transfer --prefix output --edit-nr --r1-in read1.fastq --r2-in read3.fastq --ru-in read2.fastq
```

It's good practice to remove the FIFOs after the program has finished:

```shell
rm read*.fastq output*.fastq
```

## For developers

To make modifications to `umi-transfer`, clone this repository, make your changes and then run the code with

```shell
cargo run -- <parameters>
```

or build the executable with

```shell
cargo build --release
```

Please make sure to activate code formatting by `rust-analyzer`.
