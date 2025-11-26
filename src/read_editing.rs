#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UMIDestination {
    Header,
    Inline,
}

// Updates the header and description of the reads accordingly
pub fn umi_to_record_header(
    input: bio::io::fastq::Record,
    umi: &[u8],
    umi_sep: Option<&String>,
    edit_nr: Option<u8>,
) -> Result<bio::io::fastq::Record, anyhow::Error> {
    let delim = umi_sep.as_ref().map(|s| s.as_str()).unwrap_or(":"); // the delimiter for the UMI
    let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
    if let Some(number) = edit_nr {
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, desc, input.seq(), input.qual());
        Ok(new_record)
    } else {
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, input.desc(), input.seq(), input.qual());
        Ok(new_record)
    }
}

// Updates the header and description of the reads accordingly
pub fn umi_to_record_seq(
    input: bio::io::fastq::Record,
    umi: &[u8],
    umi_qual: &[u8],
    edit_nr: Option<u8>,
) -> Result<bio::io::fastq::Record, anyhow::Error> {
    let mut concatenated_seq_str = String::with_capacity(umi.len() + input.seq().len());
    concatenated_seq_str.push_str(std::str::from_utf8(umi)?);
    concatenated_seq_str.push_str(std::str::from_utf8(input.seq())?);

    let mut concatenated_qual_str = String::with_capacity(umi_qual.len() + input.qual().len());
    concatenated_qual_str.push_str(std::str::from_utf8(umi_qual)?);
    concatenated_qual_str.push_str(std::str::from_utf8(input.qual())?);

    if let Some(number) = edit_nr {
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        // Unnecessary conversion to bytes and back to String, but Record::new() does
        // not take arguments and the fields of struct `bio::io::fastq::Record` are private,
        // so I can't implement another method to create a new record.
        let new_record = bio::io::fastq::Record::with_attrs(
            input.id(),
            desc,
            concatenated_seq_str.as_bytes(),
            concatenated_qual_str.as_bytes(),
        );
        Ok(new_record)
    } else {
        let new_record = bio::io::fastq::Record::with_attrs(
            input.id(),
            input.desc(),
            concatenated_seq_str.as_bytes(),
            concatenated_qual_str.as_bytes(),
        );
        Ok(new_record)
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_umi_to_record_header_with_edits() {
        let input = bio::io::fastq::Record::with_attrs(
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
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
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031_ACCAGCTA"
        );
        assert_eq!(result.desc(), Some("5:N:0:GCTTCAGGGT+AAGGTAGCGT"));
        assert_eq!(result.seq(), b"TCGTTTTCCGC");
        assert_eq!(result.qual(), b"FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_header_plain() {
        let input = bio::io::fastq::Record::with_attrs(
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_sep = ":".to_string();

        let result = umi_to_record_header(input, umi, Some(&umi_sep), None).unwrap();
        assert_eq!(
            result.id(),
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031:ACCAGCTA"
        );
        assert_eq!(result.desc(), Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"));
        assert_eq!(result.seq(), b"TCGTTTTCCGC");
        assert_eq!(result.qual(), b"FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_seq_with_edit_nr() {
        let input = bio::io::fastq::Record::with_attrs(
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_qual = b"########";
        let edit_nr = Some(5);

        let result = umi_to_record_seq(input, umi, umi_qual, edit_nr).unwrap();
        assert_eq!(result.id(), "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031");
        assert_eq!(result.desc(), Some("5:N:0:GCTTCAGGGT+AAGGTAGCGT"));
        assert_eq!(result.seq(), b"ACCAGCTATCGTTTTCCGC");
        assert_eq!(result.qual(), b"########FFFFFFFFFFF");
    }

    #[test]
    fn test_umi_to_record_seq_without_edit_nr() {
        let input = bio::io::fastq::Record::with_attrs(
            "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031",
            Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"),
            b"TCGTTTTCCGC",
            b"FFFFFFFFFFF",
        );
        let umi = b"ACCAGCTA";
        let umi_qual = b"########";

        let result = umi_to_record_seq(input, umi, umi_qual, None).unwrap();
        assert_eq!(result.id(), "@SCILIFELAB:500:NGISTLM:1:1101:2446:1031");
        assert_eq!(result.desc(), Some("1:N:0:GCTTCAGGGT+AAGGTAGCGT"));
        assert_eq!(result.seq(), b"ACCAGCTATCGTTTTCCGC");
        assert_eq!(result.qual(), b"########FFFFFFFFFFF");
    }
}
