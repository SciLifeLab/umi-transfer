# umi-transfer
A tool for transfering Unique Molecular Identifiers (UMIs).

The UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the
header of the first two fastq files.

## Installation
TODO
## Usage

The tool requires an input as follows:

```bash
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

Running the tool can be done by `cargo run --release -- [options] --r1-in 'fastq' <Subcommands> `, where the `--release` flag is optional, but will ensure an optimized build. <br>


### Example

```shell
cargo run --release -- --prefix 'output' --r1-in 'R1.fastq' --r2-in 'R3.fastq' separate --ru-in 'R2.fastq'
```

### Special flags

`--edit-nr` This flag will automatically change the '3' in the R3 files record-headers. Its disabled by default.
`--no-gzip` This flag diables automatic compression (.gz) of output files.

## For developers

Go to the directory with the tool and type in `cargo build` .
