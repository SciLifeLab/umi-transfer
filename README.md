
![umi-transfer](docs/img/logoheader.svg)

# umi-transfer

<p>
    <b>A command line tool for transferring Unique Molecular Identifiers (UMIs) provided as separate FastQ file to the header of records in paired FastQ files.</b>
</p>

<hr>

- [Background on Unique Molecular Identifiers](#background)
- [Installing `umi-transfer`](#installation)
- [Using `umi-transfer` to integrate UMIs](#usage)
- [Improving performance with external multi-threaded compression](#high-performance-guide)
- [Contributing bugfixes and new features](#contribution-guide-for-developers)

<hr>

## Background

To increase the accuracy of quantitative DNA sequencing experiments, Unique Molecular Identifiers may be used. UMIs are short sequences used to uniquely tag each molecule in a sample library, enabling precise identification of read duplicates. They must be added during library preparation and prior to sequencing, therefore require appropriate arrangements with your sequencing provider.

Most tools capable of taking UMIs into consideration during an analysis workflow, expect the respective UMI sequence to be embedded into the read's ID. Please consult your tools' manuals regarding the exact specification.

For some library preparation kits and sequencing adapters, the UMI sequence needs to be read together with the index from the antisense strand. Consequently, it will be output as a separate FastQ file during the demultiplexing process.

This tool efficiently integrates these separate UMIs into the headers and can also correct divergent read numbers back to the canonical `1` and `2`.

## Installation

### Binary Installation

Binaries for `umi-transfer` are available for most platforms and can be obtained from the [Releases page on GitHub](https://github.com/SciLifeLab/umi-transfer/releases). Simply navigate to the Releases page and download the appropriate binary of a release for your operating system. Once downloaded, you can place it in a directory of your choice and [optionally add the binary to your system's `$PATH`](https://astrobiomike.github.io/unix/modifying_your_path).

### Bioconda

 `umi-transfer` is also available on [BioConda](https://bioconda.github.io/). Please refer to the [Bioconda documentation](https://bioconda.github.io/recipes/umi-transfer/README.html#package-umi-transfer) for comprehensive installation instructions. If you are already familiar with conda and BioConda, here’s a quick reference:

```shell
mamba install umi-transfer
```

If you wish to create a separate virtual environment for the tool, replace `<myenvname>` with a suitable environment name of your choice and run

```shell
mamba create --name <myenvname> umi-transfer
```

### Containerized execution (Docker)

Docker provides a platform for packaging software into self-contained units called containers. Containers encapsulate all the dependencies and libraries needed to run an application, making it easy to deploy and run the software consistently across different environments.

To use `umi-transfer` with Docker, you can _pull_ the pre-made Docker image from Docker Hub. Open a terminal or command prompt and run the following command:

```shell
docker pull mzscilifelab/umi-transfer:latest
```

Once the image is downloaded, you can run `umi-transfer` within a Docker container using:

```shell
docker run -t -v `pwd`:`pwd` -w `pwd` mzscilifelab/umi-transfer:latest umi-transfer --help
```

A complete command might look like the example below. The options `-t -v -w` to Docker will ensure that your local directory is mapped to and available inside the container. Everything after the image command resembles the standard command line syntax:

```shell
docker run -t -v `pwd`:`pwd` -w `pwd` mzscilifelab/umi-transfer:latest umi-transfer external --in=read1.fq --in2=read2.fq --umi=umi.fq
```

Optionally, you can create an alias for the Docker part of the command to be able to use the containerized version as if it was locally installed. Add the line below to your `~/.profile`, `~/.bash_aliases`, `~/.bashrc` or `~/.zprofile` (depending on the terminal & configuration being used).

```shell
alias umi-transfer="docker run -t -v `pwd`:`pwd` -w `pwd` mzscilifelab/umi-transfer:latest umi-transfer"
```

### Compile from source

Given that you have [rust installed](https://www.rust-lang.org/tools/install) on your computer, download this repository and run

```shell
cargo build --release
```

That should create an executable `target/release/umi-transfer` that can be placed anywhere in your `$PATH` or be executed directly by specifying its path:

```shell
./target/release/umi-transfer --version
umi-transfer 1.0.0
```

## Usage

>### Performance Note
>
>The decompression and compression used within umi-transfer is single-threaded, so to get the most reads per minute performance, see the [high performance guide](#high-performance-guide)

The tool requires three FastQ files as input. You can manually specify the names and location of the output files with `--out` and `--out2` or the tool will append a `with_UMI` suffix to your input file names as output. It additionally accepts to choose a custom UMI delimiter with `--delim` and to set the flags `-f`, `-c` and `-z`.

`-c` is used to ensure the canonical `1` and `2` of paired files as read numbers in the output, regardless of the read numbers of the input reads. `-f` / `--force` will overwrite existing output files without prompting the user and `-c` enables the internal single-threaded compression of the output files. Alternatively, you can also specify an output file name with `.gz` suffix to obtain compressed output.

```raw
$ umi-transfer external --help
    umi-transfer-external
Integrate UMIs from a separate FastQ file

USAGE:
    umi-transfer external [OPTIONS] --in <R1_IN> --in2 <R2_IN> --umi <RU_IN>

OPTIONS:
    -c, --correct_numbers    Read numbers will be altered to ensure the canonical read numbers 1 and 2 in output file sequence headers.

    -d, --delim <DELIM>      Delimiter to use when joining the UMIs to the read name. Defaults to `:`.

    -f, --force <FORCE>      Overwrite existing output files without further warnings or prompts.

    -h, --help               Print help information
        --in <R1_IN>         [REQUIRED] Input file 1 with reads.


        --in2 <R2_IN>        [REQUIRED] Input file 2 with reads.


        --out <R1_OUT>       Path to FastQ output file for R1.


        --out2 <R2_OUT>      Path to FastQ output file for R2.


    -u, --umi <RU_IN>        [REQUIRED] Input file with UMI.

    -z, --gzip               Compress output files. By default, turned off in favour of external compression.
```

### Example

A run with just the mandatory arguments may look like this:

```shell
umi-transfer external -fz -d '_' --in 'R1.fastq' --in2 'R3.fastq' --umi 'R2.fastq'
```

`umi-transfer` warrants paired input files. To run on singletons, use the same input twice and redirect one output to `/dev/null`:

```shell
umi-transfer external --in read1.fastq --in2 read1.fastq --umi read2.fastq --out output1.fastq --out2 /dev/null
```

### High Performance Guide

The performance bottleneck of UMI integration is output file compression. [Parallel Gzip](https://github.com/madler/pigz) can be used on modern multi-processor, multi-core machines to significantly outclass the single-threaded compression that ships with `umi-transfer`.

We recommend using Unix FIFOs (First In, First Out buffered pipes) to combine `umi-transfer` and `pigz` on GNU/Linux and MacOS operating systems:

```shell
mkfifo read1.fastq
mkfifo read2.fastq
mkfifo read3.fastq
```

Assuming your compressed input files are called `read1.fastq.gz` and `read2.fastq.gz` and `read3.fastq.gz`, each can be linked to its respective FIFO like so:

```shell
$ pigz -dc read1.fastq.gz > read1.fastq &
[1] 233387
$ pigz -dc read2.fastq.gz > read2.fastq &
[2] 233388
$ pigz -dc read3.fastq.gz > read3.fastq &
[3] 233389
```

Note the trailing `&` to leave these processes running in the background. Since multi-threading is hardly helpful for decompression, you could also use `zcat` or `gzip -dc` instead of `pigz -dc` here.

We can inspect the directory with `ls` to list the compressed files and the created FIFOs:

```shell
$ ls -lh
total 1.5K
-rw-rw----. 1 alneberg ngisweden 4.5G Apr 13 12:18 read1.fastq.gz
-rw-rw----. 1 alneberg ngisweden 1.1G Apr 13 12:18 read2.fastq.gz
-rw-rw----. 1 alneberg ngisweden 4.5G Apr 13 12:18 read3.fastq.gz
prw-rw-r--. 1 alneberg ngisweden    0 Apr 13 12:46 read1.fastq
prw-rw-r--. 1 alneberg ngisweden    0 Apr 13 12:46 read2.fastq
prw-rw-r--. 1 alneberg ngisweden    0 Apr 13 12:46 read3.fastq
```

We continue to create FIFOs for the output files:

```shell
mkfifo output1.fastq
mkfifo output2.fastq
```

and set-up a multi-threaded `pigz` compression process each:

```shell
$ pigz -p 10 -c > output1.fastq.gz < output1.fastq &
[4] 233394
$ pigz -p 10 -c > output2.fastq.gz < output2.fastq &
[5] 233395
```

The argument `-p 10` specifies the number of threads that each `pigz` processes may use. The optimal setting is hardware-specific and will require some testing.

Finally, we can then run `umi-transfer` using the FIFOs like so:

```shell
umi-transfer external --in read1.fastq --in2 read3.fastq --umi read2.fastq --out output1.fastq --out2 output2.fastq
```

It's good practice to remove the FIFOs after the program has finished:

```shell
rm read1.fastq read2.fastq read3.fastq output1.fastq output2.fastq
```

## Contribution guide for developers

`umi-transfer` is a free and open-source software developed and maintained by scientists of the [Swedish National Genomics Infrastructure](https://ngisweden.scilifelab.se). We gladly welcome suggestions for improvement, bug reports and code contributions.

If you'd like to contribute code, the best way to get started is to create a personal fork of the repository. Subsequently, use a new branch to develop your feature or contribute your bug fix. Ideally, use a code linter like `rust-analyzer` in your code editor and run the tests with `cargo test`.

Before developing a new feature, we recommend opening an issue on the main repository to discuss your proposal upfront. Once you're ready, simply open a pull request to the `dev` branch and we'll happily review your changes. Thanks for your interest in contributing to `umi-transfer`!
