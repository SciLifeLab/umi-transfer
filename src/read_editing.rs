use crate::record::OwnedRecord;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UMIDestination {
    Header,
    Inline,
}

/// Append the UMI sequence to the read identifier, optionally correcting the read number.
pub fn umi_to_record_header(
    input: OwnedRecord,
    umi: &[u8],
    umi_sep: Option<&String>,
    edit_nr: Option<u8>,
) -> Result<OwnedRecord, anyhow::Error> {
    let delim = umi_sep.map(|s| s.as_str()).unwrap_or(":");
    let id_part = input.id();

    let mut new_head =
        Vec::with_capacity(id_part.len() + delim.len() + umi.len() + input.head.len());
    new_head.extend_from_slice(id_part);
    new_head.extend_from_slice(delim.as_bytes());
    new_head.extend_from_slice(umi);

    if let Some(desc) = input.desc() {
        new_head.push(b' ');
        if let Some(number) = edit_nr {
            let mut new_desc = desc.to_vec();
            if !new_desc.is_empty() {
                new_desc[0] = b'0' + number;
            }
            new_head.extend_from_slice(&new_desc);
        } else {
            new_head.extend_from_slice(desc);
        }
    }

    Ok(OwnedRecord::new(new_head, input.seq, input.qual))
}

/// Prepend the UMI sequence (and its quality scores) to the read sequence, optionally
/// correcting the read number in the description.
pub fn umi_to_record_seq(
    input: OwnedRecord,
    umi: &[u8],
    umi_qual: &[u8],
    edit_nr: Option<u8>,
) -> Result<OwnedRecord, anyhow::Error> {
    let mut new_seq = Vec::with_capacity(umi.len() + input.seq.len());
    new_seq.extend_from_slice(umi);
    new_seq.extend_from_slice(&input.seq);

    let mut new_qual = Vec::with_capacity(umi_qual.len() + input.qual.len());
    new_qual.extend_from_slice(umi_qual);
    new_qual.extend_from_slice(&input.qual);

    let new_head = if let Some(number) = edit_nr {
        if let Some(desc) = input.desc() {
            let id_part = input.id();
            let mut h = Vec::with_capacity(id_part.len() + 1 + desc.len());
            h.extend_from_slice(id_part);
            h.push(b' ');
            let mut new_desc = desc.to_vec();
            if !new_desc.is_empty() {
                new_desc[0] = b'0' + number;
            }
            h.extend_from_slice(&new_desc);
            h
        } else {
            input.head
        }
    } else {
        input.head
    };

    Ok(OwnedRecord::new(new_head, new_seq, new_qual))
}

#[cfg(test)]
mod tests {

    use super::*;

    fn make_record(id: &str, desc: Option<&str>, seq: &[u8], qual: &[u8]) -> OwnedRecord {
        let head = match desc {
            Some(d) => {
                let mut h = id.as_bytes().to_vec();
                h.push(b' ');
                h.extend_from_slice(d.as_bytes());
                h
            }
            None => id.as_bytes().to_vec(),
        };
        OwnedRecord::new(head, seq.to_vec(), qual.to_vec())
    }

    #[test]
    fn test_umi_to_record_header_with_edits() {
        let input = make_record(
            "SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_sep = "_".to_string();
        let edit_nr = Some(5);

        let result = umi_to_record_header(input, umi, Some(&umi_sep), edit_nr).unwrap();
        assert_eq!(
            result.id(),
            b"SCILIFELAB:500:NGISTLM:1:1101:2446:1031_ACCAGCTA"
        );
        assert_eq!(result.desc(), Some(b"5:N:0:GCTTCAGGGT+AAGGTAGCGT".as_ref()));
        assert_eq!(result.seq, b"TCGTTTTCCGC");
        assert_eq!(result.qual, b"FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_header_plain() {
        let input = make_record(
            "SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_sep = ":".to_string();

        let result = umi_to_record_header(input, umi, Some(&umi_sep), None).unwrap();
        assert_eq!(
            result.id(),
            b"SCILIFELAB:500:NGISTLM:1:1101:2446:1031:ACCAGCTA"
        );
        assert_eq!(result.desc(), Some(b"1:N:0:GCTTCAGGGT+AAGGTAGCGT".as_ref()));
        assert_eq!(result.seq, b"TCGTTTTCCGC");
        assert_eq!(result.qual, b"FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_seq_with_edit_nr() {
        let input = make_record(
            "SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_qual = b"########";
        let edit_nr = Some(5);

        let result = umi_to_record_seq(input, umi, umi_qual, edit_nr).unwrap();
        assert_eq!(result.id(), b"SCILIFELAB:500:NGISTLM:1:1101:2446:1031");
        assert_eq!(result.desc(), Some(b"5:N:0:GCTTCAGGGT+AAGGTAGCGT".as_ref()));
        assert_eq!(result.seq, b"ACCAGCTATCGTTTTCCGC");
        assert_eq!(result.qual, b"########FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_seq_without_edit_nr() {
        let input = make_record(
            "SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_qual = b"########";

        let result = umi_to_record_seq(input, umi, umi_qual, None).unwrap();
        assert_eq!(result.id(), b"SCILIFELAB:500:NGISTLM:1:1101:2446:1031");
        assert_eq!(result.desc(), Some(b"1:N:0:GCTTCAGGGT+AAGGTAGCGT".as_ref()));
        assert_eq!(result.seq, b"ACCAGCTATCGTTTTCCGC");
        assert_eq!(result.qual, b"########FFFFFFFFFFF");
    }
}
