#[derive(clap::ValueEnum, Clone,Debug)]
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

    let mut concatenated_seq = Vec::with_capacity(input.seq().len() + umi.len());
    concatenated_seq.extend_from_slice(umi);
    concatenated_seq.extend_from_slice(input.seq());
    let concatenated_seq_str = String::from_utf8(concatenated_seq).unwrap();

    let mut concatenated_qual = Vec::with_capacity(input.qual().len() + umi_qual.len());
    concatenated_qual.extend_from_slice(umi);
    concatenated_qual.extend_from_slice(input.seq());
    let concatenated_qual_str = String::from_utf8(concatenated_qual).unwrap();

    if let Some(number) = edit_nr {
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        // ToDo: unnecessary conversion to bytes and back to String, but Record::new() does not take arguments
        let new_record =  
        bio::io::fastq::Record::with_attrs(input.id(), desc, concatenated_seq_str.as_bytes(), concatenated_qual_str.as_bytes());
        Ok(new_record)
    } else {
        let new_record =
            bio::io::fastq::Record::with_attrs(input.id(), input.desc(),  concatenated_seq_str.as_bytes(), concatenated_qual_str.as_bytes());
        Ok(new_record)
    }
}