use anyhow::{Error};


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
    if let Some(number) = edit_nr {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, desc, input.seq(), input.qual());
        Ok(new_record)
    } else {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, input.desc(), input.seq(), input.qual());
        Ok(new_record)
    }
}


// Updates the header and description of the reads accordingly
pub fn umi_to_record_seq(
    input: bio::io::fastq::Record,
    umi: &[u8],
    umi_sep: Option<&String>,
    edit_nr: Option<u8>,
) -> Result<bio::io::fastq::Record, anyhow::Error> {
    let delim = umi_sep.as_ref().map(|s| s.as_str()).unwrap_or(":"); // the delimiter for the UMI
    if let Some(number) = edit_nr {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, desc, input.seq(), input.qual());
        Ok(new_record)
    } else {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, input.desc(), input.seq(), input.qual());
        Ok(new_record)
    }
}