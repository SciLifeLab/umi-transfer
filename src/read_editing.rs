use std::io::{self, Write};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UMIDestination {
    Header,
    Inline,
}

/// The read id is the header (bytes after `@`, no newline) up to the first
/// space.
pub fn read_id(head: &[u8]) -> &[u8] {
    split_head(head).0
}

/// Split a header into its id and the optional ` description` remainder.
fn split_head(head: &[u8]) -> (&[u8], Option<&[u8]>) {
    match head.iter().position(|&b| b == b' ') {
        Some(i) => (&head[..i], Some(&head[i + 1..])),
        None => (head, None),
    }
}

/// Write the header line `@id[<delim><umi>][ desc]\n`. With `read_nr` set
/// (the `--correct_numbers` flag) the first byte of the description is replaced
/// by the read-number digit. A header with no description and `read_nr` set is
/// left unchanged (there is no read-number field to correct).
fn write_header(
    w: &mut impl Write,
    id: &[u8],
    desc: Option<&[u8]>,
    umi_in_header: Option<(&[u8], &[u8])>,
    read_nr: Option<u8>,
) -> io::Result<()> {
    w.write_all(b"@")?;
    w.write_all(id)?;
    if let Some((delim, umi)) = umi_in_header {
        w.write_all(delim)?;
        w.write_all(umi)?;
    }
    if let Some(desc) = desc {
        w.write_all(b" ")?;
        match read_nr {
            Some(n) if !desc.is_empty() => {
                // Read numbers are single digits (1 or 2).
                debug_assert!(n < 10, "read number must be a single digit");
                w.write_all(&[b'0' + n])?;
                w.write_all(&desc[1..])?;
            }
            _ => w.write_all(desc)?,
        }
    }
    w.write_all(b"\n")
}

/// Header mode: splice the UMI into the header; sequence and quality pass
/// through unchanged. Every slice borrows into the input or UMI buffer, so the
/// only copy is the writer's own buffering.
pub fn write_umi_in_header(
    w: &mut impl Write,
    head: &[u8],
    seq: &[u8],
    qual: &[u8],
    umi: &[u8],
    delim: &[u8],
    read_nr: Option<u8>,
) -> io::Result<()> {
    let (id, desc) = split_head(head);
    write_header(w, id, desc, Some((delim, umi)), read_nr)?;
    w.write_all(seq)?;
    w.write_all(b"\n+\n")?;
    w.write_all(qual)?;
    w.write_all(b"\n")
}

/// Inline mode: prepend the UMI to sequence and quality; the header keeps its
/// id and description. The UMI is written as a leading segment of each.
pub fn write_umi_inline(
    w: &mut impl Write,
    head: &[u8],
    seq: &[u8],
    qual: &[u8],
    umi_seq: &[u8],
    umi_qual: &[u8],
    read_nr: Option<u8>,
) -> io::Result<()> {
    let (id, desc) = split_head(head);
    write_header(w, id, desc, None, read_nr)?;
    w.write_all(umi_seq)?;
    w.write_all(seq)?;
    w.write_all(b"\n+\n")?;
    w.write_all(umi_qual)?;
    w.write_all(qual)?;
    w.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(f: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        f(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    const HEAD: &[u8] = b"SCILIFELAB:500:NGISTLM:1:1101:2446:1031 1:N:0:GCTTCAGGGT+AAGGTAGCGT";

    #[test]
    fn header_with_read_number() {
        let out = render(|w| write_umi_in_header(w, HEAD, b"TCGTTTTCCGC", b"FFFFFFFFFFF", b"ACCAGCTA", b"_", Some(5)));
        assert_eq!(
            out,
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031_ACCAGCTA 5:N:0:GCTTCAGGGT+AAGGTAGCGT\nTCGTTTTCCGC\n+\nFFFFFFFFFFF\n"
        );
    }

    #[test]
    fn header_plain() {
        let out = render(|w| write_umi_in_header(w, HEAD, b"TCGTTTTCCGC", b"FFFFFFFFFFF", b"ACCAGCTA", b":", None));
        assert_eq!(
            out,
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031:ACCAGCTA 1:N:0:GCTTCAGGGT+AAGGTAGCGT\nTCGTTTTCCGC\n+\nFFFFFFFFFFF\n"
        );
    }

    #[test]
    fn inline_with_read_number() {
        let out = render(|w| write_umi_inline(w, HEAD, b"TCGTTTTCCGC", b"FFFFFFFFFFF", b"ACCAGCTA", b"########", Some(5)));
        assert_eq!(
            out,
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031 5:N:0:GCTTCAGGGT+AAGGTAGCGT\nACCAGCTATCGTTTTCCGC\n+\n########FFFFFFFFFFF\n"
        );
    }

    #[test]
    fn inline_plain() {
        let out = render(|w| write_umi_inline(w, HEAD, b"TCGTTTTCCGC", b"FFFFFFFFFFF", b"ACCAGCTA", b"########", None));
        assert_eq!(
            out,
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031 1:N:0:GCTTCAGGGT+AAGGTAGCGT\nACCAGCTATCGTTTTCCGC\n+\n########FFFFFFFFFFF\n"
        );
    }
}
