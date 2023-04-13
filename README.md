# umi-transfer
A tool for transfering Unique Molecular Identifiers (UMIs).

The UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the
header of the first two fastq files.

## Installation

### Compile from source
Given that you have [rust installed](https://www.rust-lang.org/tools/install) on your computer, download this repo and run
```shell
cargo build --release
```
That should create an executable `target/release/umi-transfer` that can be placed anywhere in your `$PATH` or be executed directly by specifying its' path:

```shell
./target/release/umi-transfer --version
umi-transfer 0.2.0
```
## Usage

>### Performance Note: 
>The decompression and compression used within umi-transfer is single-threaded, so to get the most reads per minute performance, see the [high performance guide](#high-performance-guide)

The tool requires three fastq files and additionally accepts flags to adjust the behaviour as can be seen from the help message:

```raw
$ umi-transfer --help
umi-transfer 0.2.0
Judit Hohenthal, Matthias Zepper, Johannes Alneberg
A tool for transfering Unique Molecular Identifiers (UMIs).

The UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the
header of the first two fastq files.


USAGE:
    umi-transfer [OPTIONS] --r1-in <R1_IN> --r2-in <R2_IN> --ru-in <RU_IN>

OPTIONS:
        --edit-nr            Automatically change '3' into '2' in sequence header of output file
                             from R3.
                             
        --gzip               Compress output files with gzip. By default turned off to encourage use
                             of external compression (see Readme).
                             
    -h, --help               Print help information
        --prefix <PREFIX>    Prefix for output files, omitted flag will result in default value.
                             
                               [default: output]
        --r1-in <R1_IN>      [REQUIRED] Input file 1 with reads.
                                 
                              
        --r2-in <R2_IN>      [REQUIRED] Input file 2 with reads.
                                 
                              
        --ru-in <RU_IN>      [REQUIRED] Input file with UMI.
                                     
    -V, --version            Print version information
```

### Example

```shell
cargo run --release -- --prefix 'output' --edit-nr --r1-in 'R1.fastq' --r2-in 'R3.fastq' --ru-in 'R2.fastq'
```

### High Performance Guide
If you have more than one thread available on your computer and would like to process the read files as quickly as possible we recommend to use unix FIFOs (First In First Out) to handle decompression and compression of the fastq files.
This can be done as follows, given that you have your input files compressed as `fastq.gz`, first create FIFOs to represent your uncompressed input files:

```shell
$ mkfifo read1.fastq
$ mkfifo read2.fastq
$ mkfifo read3.fastq
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
$ umi-transfer --prefix output --edit-nr --r1-in read1.fastq --r2-in read3.fastq --ru-in read2.fastq
```

It's good practice to remove the FIFOs after the program has finished:
```shell
rm read*.fastq output*.fastq
```
## For developers

Go to the directory with the tool and type in `cargo build` .
