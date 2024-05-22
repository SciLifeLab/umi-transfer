
![umi-transfer](docs/img/logoheader.svg)

# umi-transfer

<p>
    <b>A command line tool for transferring Unique Molecular Identifiers (UMIs) provided as separate FastQ file to the header of records in paired FastQ files.</b>
</p>

<hr>

- [Background on Unique Molecular Identifiers](#background)
- [Installing `umi-transfer`](#installation)
- [Using `umi-transfer` to integrate UMIs](#usage)
- [Chaining with other software](#chaining-with-other-software)
- [Contributing bugfixes and new features](#contribution-guide-for-developers)

<hr>

[![License: MIT](https://img.shields.io/badge/License-MIT-491f53.svg)](https://opensource.org/licenses/MIT)
![GitHub Actions Tests](https://img.shields.io/github/actions/workflow/status/SciLifeLab/umi-transfer/.github%2Fworkflows%2Ftesting.yml?branch=dev&logo=github&label=Tests&color=%23a7c947)
[![codecov](https://codecov.io/gh/SciLifeLab/umi-transfer/branch/dev/graph/badge.svg)](https://codecov.io/gh/SciLifeLab/umi-transfer)
[![Build status](https://img.shields.io/github/actions/workflow/status/SciLifeLab/umi-transfer/.github%2Fworkflows%2Frelease.yml?branch=dev&label=Binary%20builds&logo=github&color=%23a7c947)](https://github.com/SciLifeLab/umi-transfer/releases/latest)
[![Docker container status](https://img.shields.io/github/actions/workflow/status/SciLifeLab/umi-transfer/.github%2Fworkflows%2Fcontainer.yml?branch=dev&label=Docker%20builds&logo=docker&color=%23a7c947)](https://hub.docker.com/r/mzscilifelab/umi-transfer)
[![Install with Bioconda](https://img.shields.io/badge/Available%20via-Bioconda-045c64.svg)](https://bioconda.github.io/recipes/umi-transfer/README.html)

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
umi-transfer 1.5.0
```

## Usage

The tool requires three FastQ files as input. You can manually specify the names and location of the output files with `--out` and `--out2` or the tool will automatically append a `with_UMI` suffix to your input file names. It additionally accepts to choose a custom UMI delimiter with `--delim` and to set the flags `-f`, `-c` and `-z`.

`-c` is used to ensure the canonical `1` and `2` of paired files as read numbers in the output, regardless of the read numbers of the input reads. `-f` / `--force` will overwrite existing output files without prompting the user and `-z` enables the internal compression of the output files. Alternatively, you can also specify an output file name with `.gz` suffix to obtain compressed output.

```raw
$ umi-transfer external --help


Integrate UMIs from a separate FastQ file

Usage: umi-transfer external [OPTIONS] --in <R1_IN> --in2 <R2_IN> --umi <RU_IN>

Options:
  -c, --correct_numbers
          Read numbers will be altered to ensure the canonical read numbers 1 and 2 in output file sequence headers.


  -z, --gzip
          Compress output files. Turned off by default.


  -l, --compression_level <COMPRESSION_LEVEL>
          Choose the compression level: Maximum 9, defaults to 3. Higher numbers result in smaller files but take longer to compress.


  -t, --threads <NUM_THREADS>
          Number of threads to use for processing. Defaults to the number of logical cores available.


  -f, --force
          Overwrite existing output files without further warnings or prompts.


  -d, --delim <DELIM>
          Delimiter to use when joining the UMIs to the read name. Defaults to `:`.


      --in <R1_IN>
          [REQUIRED] Input file 1 with reads.


      --in2 <R2_IN>
          [REQUIRED] Input file 2 with reads.


  -u, --umi <RU_IN>
          [REQUIRED] Input file with UMI.


      --out <R1_OUT>
          Path to FastQ output file for R1.


      --out2 <R2_OUT>
          Path to FastQ output file for R2.


  -h, --help
          Print help
  -V, --version
          Print version
```

### Example

A typical run may look like this:

```shell
umi-transfer external -fz -d '_' --in 'R1.fastq' --in2 'R3.fastq' --umi 'R2.fastq'
```

`umi-transfer` warrants paired input files. To run on singletons, use the same input twice and redirect one output to `/dev/null`:

```shell
umi-transfer external --in read1.fastq --in2 read1.fastq --umi read2.fastq --out output1.fastq --out2 /dev/null
```

### Chaining with other software

`umi-transfer` cannot be used with the pipe operator, because it neither supports writing output to `stdout` nor reading input from `stdin`. However, FIFOs (_First In, First Out buffered pipes_) can be used to elegantly combine `umi-transfer` with other software on GNU/Linux and MacOS operating systems.

For example, we may want to use external compression software like [Parallel Gzip](https://github.com/madler/pigz) together with `umi-transfer`. For this purpose, it would be unfavorable to write the data uncompressed to disk before compressing it. Instead, we create named pipes with `mkfifo`, which can be provided to `umi-transfer` as if they were regular output file paths. In reality, the data is directly passed on to `pigz` via a buffered stream.

First, the named pipes are created:

```shell
mkfifo output1
mkfifo output2
```

Then a multi-threaded `pigz` compression is tied to the FIFO. Note the trailing `&` to leave these processes running in the background.

```shell
$ pigz -p 10 -c > output1.fastq.gz < output1 &
[4] 233394
$ pigz -p 10 -c > output2.fastq.gz < output2 &
[5] 233395
```

The argument `-p 10` specifies the number of threads that each `pigz` processes may use. The optimal setting is hardware-specific and will require some testing.

Finally, we can run `umi-transfer` using the FIFOs as output paths:

```shell
umi-transfer external --in read1.fastq --in2 read3.fastq --umi read2.fastq --out output1 --out2 output2
```

It's good practice to remove the FIFOs after the program has finished:

```shell
rm output1.fastq output2.fastq
```

## Contribution guide for developers

`umi-transfer` is a free and open-source software developed and maintained by scientists of the [Swedish National Genomics Infrastructure](https://ngisweden.scilifelab.se). We gladly welcome suggestions for improvement, bug reports and code contributions.

If you'd like to contribute code, the best way to get started is to create a personal fork of the repository. Subsequently, use a new branch to develop your feature or contribute your bug fix. Ideally, use a code linter like `rust-analyzer` in your code editor and run the tests with `cargo test`.

Before developing a new feature, we recommend opening an issue on the main repository to discuss your proposal upfront. Once you're ready, simply open a pull request to the `dev` branch and we'll happily review your changes. Thanks for your interest in contributing to `umi-transfer`!
