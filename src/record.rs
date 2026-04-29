use std::io::Write;

/// An owned FASTQ record with no lifetime dependency on any read buffer.
///
/// `head` holds the full header line content **after** the leading `@`
/// (e.g. `"READ1 1:N:0:SAMPLE"`).  The `@` is added back when writing.
#[derive(Clone, Debug)]
pub struct OwnedRecord {
    /// Full header line after `@`, including any space-separated description.
    pub head: Vec<u8>,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

impl OwnedRecord {
    pub fn new(head: Vec<u8>, seq: Vec<u8>, qual: Vec<u8>) -> Self {
        Self { head, seq, qual }
    }

    /// The ID portion of the header (up to the first space, or the whole `head`).
    #[inline]
    pub fn id(&self) -> &[u8] {
        match self.head.iter().position(|&b| b == b' ') {
            Some(i) => &self.head[..i],
            None => &self.head,
        }
    }

    /// The description portion of the header (after the first space), if present.
    #[inline]
    pub fn desc(&self) -> Option<&[u8]> {
        match self.head.iter().position(|&b| b == b' ') {
            Some(i) => Some(&self.head[i + 1..]),
            None => None,
        }
    }

    /// Write this record in FASTQ format to any [`Write`] target.
    pub fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"@")?;
        writer.write_all(&self.head)?;
        writer.write_all(b"\n")?;
        writer.write_all(&self.seq)?;
        writer.write_all(b"\n+\n")?;
        writer.write_all(&self.qual)?;
        writer.write_all(b"\n")
    }
}
