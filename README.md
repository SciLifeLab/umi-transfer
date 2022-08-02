# Building

Go to the directory with the tool and type in `cargo build` .

# Running

### Usage

The tool requires an input as follows:

> `umi-transfer [OPTIONS] <SUBCOMMAND> `<br>

`OPTIONS:`
| Flag | Required | Description |
| ------------- | :-----------: | ----------: |
| `-h`,`--help` | No | Print help information |
| `--prefix` | No, but default will be '`integrated`' | dictates name of output files|
| `--r1-in` | Yes | FASTQ file with reads|
| `--r2-in` | No | FASTQ file with reads |

`SUBCOMMANDS: `

> `inline:`
>
> > | Flag         |          Required          |                Description |
> > | ------------ | :------------------------: | -------------------------: |
> > | `--pattern1` |            Yes             | Nucleotide Pattern for UMI |
> > | `--pattern2` | Needed if `--r2-in` exists | Nucleotide Pattern for UMI |
>
> `separate:`
>
> > | Flag      | Required |                  Description |
> > | --------- | :------: | ---------------------------: |
> > | `--ru-in` |   Yes    | FASTQ containing UMI records |

Running the tool can be done by `cargo run --release -- [options] --r1-in 'fastq' <Subcommands> `, where the `--release` flag is optional, but will ensure an optimized build. <br>

### Inline UMI extraction example:

`cargo run --release -- --prefix 'output' --r1-in 'R1.fastq' --r2-in 'R2.fastq' inline --pattern1 'NNNNNNNNN' --pattern2 'NNNNNNNNN'`

### UMI in seperate file example:

`cargo run --release -- --prefix 'output' --r1-in 'R1.fastq' --r2-in 'R3.fastq' separate --ru-in 'R2.fastq'`
