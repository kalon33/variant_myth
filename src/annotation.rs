//! Struct to store annotation

/* std use */

/* crate use */
use bstr::ByteSlice as _;

/* project use */
use crate::error;

/// Define annotation strand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strand {
    /// Annotation is in forward strand
    Forward,
    /// Annotation is in reverse strand
    Reverse,
}

impl std::fmt::Display for Strand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Strand::Forward => write!(f, "+"),
            Strand::Reverse => write!(f, "-"),
        }
    }
}

/// Define annotation frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// Annotation frame is Unknow
    Unknow,
    /// Annotation frame is 0
    Zero,
    /// Annotation frame is 1
    One,
    /// Annotation frame is 2
    Two,
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Frame::Unknow => write!(f, "."),
            Frame::Zero => write!(f, "0"),
            Frame::One => write!(f, "1"),
            Frame::Two => write!(f, "2"),
        }
    }
}

/// Store attribute of gff record
#[derive(Debug, Clone, std::default::Default, PartialEq, Eq)]
pub struct Attribute {
    pub(crate) id: Vec<u8>,     // ID=
    pub(crate) name: Vec<u8>,   // Name=
    pub(crate) parent: Vec<u8>, // Parent=
}

impl Attribute {
    /// Create an attribute from u8 slice
    pub fn from_u8_slice(slice: &[u8]) -> error::Result<Self> {
        let mut obj = Attribute::default();

        if slice.is_empty() {
            return Ok(obj);
        }

        for attribute in slice.split_str(";") {
            match attribute {
                [b'I', b'D', b'=', value @ ..] => obj.id = value.to_vec(),
                [b'N', b'a', b'm', b'e', b'=', value @ ..] => obj.name = value.to_vec(),
                [b'P', b'a', b'r', b'e', b'n', b't', b'=', value @ ..] => {
                    obj.parent = value.to_vec()
                }
                _ => {}
            }
        }

        Ok(obj)
    }

    /// Get gene name
    pub fn get_id(&self) -> &[u8] {
        &self.id
    }

    /// Get transcript_id
    pub fn get_name(&self) -> &[u8] {
        &self.name
    }

    /// Get exon_number
    pub fn get_parent(&self) -> &[u8] {
        &self.parent
    }

    /// Get exon_number
    pub fn set_parent(&mut self, value: Vec<u8>) {
        self.parent = value
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut fields = vec![];

        unsafe {
            if !self.id.is_empty() {
                fields.push(format!(
                    "ID={}",
                    String::from_utf8_unchecked(self.id.to_vec())
                ));
            }
            if !self.name.is_empty() {
                fields.push(format!(
                    "Name={}",
                    String::from_utf8_unchecked(self.name.to_vec())
                ));
            }
            if !self.parent.is_empty() {
                fields.push(format!(
                    "Parent={}",
                    String::from_utf8_unchecked(self.parent.to_vec())
                ));
            }
        }

        write!(f, "{}", fields.join(";"))
    }
}

/// Wrapper around f64 to store annotation Score
#[derive(Debug, Clone)]
pub struct Score(pub f64);

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_infinite() && other.0.is_infinite() {
            true
        } else {
            (self.0 - other.0).abs() < f64::EPSILON
        }
    }
}

impl Eq for Score {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Store information of a Gff3 field
pub struct Annotation {
    pub(crate) seqname: Vec<u8>,
    pub(crate) source: Vec<u8>,
    pub(crate) feature: Vec<u8>,
    pub(crate) start: u64,
    pub(crate) stop: u64,
    pub(crate) score: Score,
    pub(crate) strand: Strand,
    pub(crate) frame: Frame,
    pub(crate) attribute: Attribute,
}

impl Annotation {
    /// Build a new annotations from a csv::ByteRecord
    pub fn from_byte_record(record: &csv::ByteRecord) -> error::Result<Self> {
        unsafe {
            Ok(Self {
                seqname: record.get(0).unwrap().to_vec(),
                source: record.get(1).unwrap().to_vec(),
                feature: record.get(2).unwrap().to_vec(),
                start: String::from_utf8_unchecked(record.get(3).unwrap().to_vec())
                    .parse::<u64>()
                    .unwrap(),
                stop: String::from_utf8_unchecked(record.get(4).unwrap().to_vec())
                    .parse::<u64>()
                    .unwrap(),
                score: match record.get(5).unwrap() {
                    b"." => Score(f64::INFINITY),
                    other => Score(String::from_utf8_unchecked(other.to_vec()).parse::<f64>()?),
                },
                strand: match record.get(6).unwrap() {
                    b"+" => Strand::Forward,
                    b"-" => Strand::Reverse,
                    b"." => Strand::Forward,
                    _ => return Err(error::Error::GffBadStrand.into()),
                },
                frame: match record.get(7).unwrap() {
                    b"." => Frame::Unknow,
                    b"0" => Frame::Zero,
                    b"1" => Frame::One,
                    b"2" => Frame::Two,
                    _ => return Err(error::Error::GffBadFrame.into()),
                },
                attribute: Attribute::from_u8_slice(record.get(8).unwrap())?,
            })
        }
    }

    /// Build annotation from another Annotation, set feature, start, stop, new feature parent is actual feature
    pub fn create_child(a: &Self, feature: &[u8], start: u64, stop: u64) -> Self {
        let mut obj = a.clone();
        obj.feature = feature.to_vec();
        obj.start = start;
        obj.stop = stop;
        obj.attribute.set_parent(a.attribute.get_id().to_vec());

        obj
    }

    /// Get start position 0-based
    pub fn get_start(&self) -> u64 {
        self.start - 1
    }

    /// Get stop position 0-based
    pub fn get_stop(&self) -> u64 {
        self.stop
    }

    /// Create interval associate with annotation 0-based
    pub fn get_interval(&self) -> core::ops::Range<u64> {
        (self.start - 1)..(self.stop)
    }

    /// Get seqname
    pub fn get_seqname(&self) -> &[u8] {
        &self.seqname
    }

    /// Get feature
    pub fn get_feature(&self) -> &[u8] {
        &self.feature
    }

    /// Get seqname
    pub fn get_source(&self) -> &[u8] {
        &self.source
    }

    /// Get strand
    pub fn get_strand(&self) -> &Strand {
        &self.strand
    }

    /// Get frame
    pub fn get_frame(&self) -> &Frame {
        &self.frame
    }

    /// Get attribute annotation
    pub fn get_attribute(&self) -> &Attribute {
        &self.attribute
    }

    /// Get parent
    pub fn get_parent(&self) -> &[u8] {
        self.attribute.get_parent()
    }

    #[cfg(test)]
    /// Generate a fake annotation with a seqname, start and stop
    pub fn test_annotation(seqname: Vec<u8>, start: u64, stop: u64) -> Self {
        Self {
            seqname,
            source: b"variant_myth".to_vec(),
            feature: b"test".to_vec(),
            start,
            stop,
            score: Score(f64::INFINITY),
            strand: Strand::Forward,
            frame: Frame::Unknow,
            attribute: Attribute::default(),
        }
    }
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        unsafe {
            write!(
                f,
                "{} {} {} {} {} {} {} {} {}",
                String::from_utf8_unchecked(self.seqname.clone()),
                String::from_utf8_unchecked(self.source.clone()),
                String::from_utf8_unchecked(self.feature.clone()),
                self.start,
                self.stop,
                self.score.0,
                self.strand,
                self.frame,
                self.attribute,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */
    use bstr::ByteSlice;

    /* project use */
    use super::*;
    use crate::test_data;

    #[test]
    fn format_strand() {
        assert_eq!(format!("{}", Strand::Forward), "+");
        assert_eq!(format!("{}", Strand::Reverse), "-");
    }

    #[test]
    fn format_frame() {
        assert_eq!(format!("{}", Frame::Unknow), ".");
        assert_eq!(format!("{}", Frame::Zero), "0");
        assert_eq!(format!("{}", Frame::One), "1");
        assert_eq!(format!("{}", Frame::Two), "2");
    }

    #[test]
    fn attribute() -> error::Result<()> {
        let slice = &test_data::GFF_CSV_RECORD[1][8];

        let attribute = Attribute::from_u8_slice(slice)?;
        assert_eq!(attribute.get_id(), b"ENST00000797271.1");
        assert_eq!(attribute.get_parent(), b"ENSG00000286586.2");
        assert_eq!(attribute.get_name(), b"transcript_name");

        assert_eq!(
            format!("{}", attribute),
            "ID=ENST00000797271.1;Name=transcript_name;Parent=ENSG00000286586.2"
        );

        let slice = b"";
        let attribute = Attribute::from_u8_slice(slice)?;
        assert_eq!(attribute.get_id(), b"");
        assert_eq!(attribute.get_parent(), b"");
        assert_eq!(attribute.get_name(), b"");

        let _result: std::result::Result<(), error::Error> = Err(
            error::Error::AttributeNameNotSupport(String::from_utf8(b"test".to_vec()).unwrap()),
        );
        assert!(matches!(Attribute::from_u8_slice(b"test"), _result));

        Ok(())
    }

    #[test]
    fn annotation() -> error::Result<()> {
        let mut data: Vec<&[u8]> = test_data::GFF_BY_LINE[1].split_str("\t").collect();

        // Basic test
        let record = csv::ByteRecord::from(data.clone());
        let annotation = Annotation::from_byte_record(&record)?;

        assert_eq!(annotation.get_interval(), 50..30235);
        assert_eq!(annotation.get_seqname(), b"chrA");
        assert_eq!(annotation.get_source(), b"HAVANA");
        assert_eq!(annotation.get_feature(), b"transcript");
        assert_eq!(annotation.get_strand(), &Strand::Forward);
        assert_eq!(annotation.get_frame(), &Frame::Unknow);
        assert_eq!(annotation.get_parent(), b"ENSG00000286586.2");
        assert_eq!(
            annotation.get_attribute(),
            test_data::GFF_ANNOTATION[1].get_attribute()
        );

        // Format
        assert_eq!(
            format!("{}", annotation),
"chrA HAVANA transcript 51 30235 inf + . ID=ENST00000797271.1;Name=transcript_name;Parent=ENSG00000286586.2"
        );

        // Change exon
        let annotation = Annotation::create_child(&annotation, b"exon", 29554, 31097);

        assert_eq!(annotation.get_feature(), b"exon");

        // All possible value for Strand
        data[6] = b"-";
        let record = csv::ByteRecord::from(data.clone());
        let annotation = Annotation::from_byte_record(&record)?;
        assert_eq!(annotation.get_strand(), &Strand::Reverse);

        // All possible value for Frame
        data[7] = b"0";
        let record = csv::ByteRecord::from(data.clone());
        let annotation = Annotation::from_byte_record(&record)?;
        assert_eq!(annotation.get_frame(), &Frame::Zero);

        data[7] = b"1";
        let record = csv::ByteRecord::from(data.clone());
        let annotation = Annotation::from_byte_record(&record)?;
        assert_eq!(annotation.get_frame(), &Frame::One);

        data[7] = b"2";
        let record = csv::ByteRecord::from(data.clone());
        let annotation = Annotation::from_byte_record(&record)?;
        assert_eq!(annotation.get_frame(), &Frame::Two);

        // Error
        data[5] = b"Not a Float";
        let record = csv::ByteRecord::from(data.clone());
        let _result = "Other not a Float".parse::<f64>();
        assert!(matches!(Annotation::from_byte_record(&record), _result));
        data[5] = b"1.1";

        data[6] = b"Forward";
        let record = csv::ByteRecord::from(data.clone());
        let _result: std::result::Result<(), error::Error> = Err(error::Error::GffBadStrand);
        assert!(matches!(Annotation::from_byte_record(&record), _result));
        data[6] = b"+";

        data[7] = b"3";
        let record = csv::ByteRecord::from(data.clone());
        let _result: std::result::Result<(), error::Error> = Err(error::Error::GffBadFrame);
        assert!(matches!(Annotation::from_byte_record(&record), _result));

        Ok(())
    }

    #[test]
    fn test_annotation() {
        let annotation = Annotation::test_annotation(b"test_annotation_test".to_vec(), 10, 110);

        // set value
        assert_eq!(annotation.get_seqname(), b"test_annotation_test");
        assert_eq!(annotation.get_start(), 9);
        assert_eq!(annotation.get_stop(), 110);

        // default value
        assert_eq!(annotation.get_source(), b"variant_myth");
        assert_eq!(annotation.get_feature(), b"test");
        assert_eq!(annotation.get_strand(), &Strand::Forward);
        assert_eq!(annotation.get_frame(), &Frame::Unknow);
    }
}
