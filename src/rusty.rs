use std::fs::File;
use std::io::{self, BufReader, Read};
use bytecount::count as byte_counter;

const READ_BUFFER_SIZE: usize = 128 * 1024;

/// Counts the number of '\n's in a file as quickly as possible and then
/// returns the count.
pub fn count_file_lines(filename: &str) -> io::Result<usize> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; READ_BUFFER_SIZE]; // 256kb at a time
    let mut count = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        count += byte_counter(&buffer[..bytes_read], b'\n');
    }

    Ok(count)
}


/// Attempts to parse a supply level reading into a number of units and a
/// level. The expected format is one of:
///     ?               => unknown (represented by -1, -1)
///     -               => zero    (0, 0),
///     <units><level>
///         units := [0-9]{1,4}
///         level := { [Ll] => 1, [Mm] => 2, [Hh] => 3, '?' => -1 }
///
pub fn parse_supply_level(reading: &str) -> Result<(i32, i32), &'static str> {
    if reading.len() > 1 {
        if !reading.as_bytes()[0].is_ascii_digit() {
            return Err("malformed supply reading");
        }
        // At least two characters, we can hope for units and a level
        // Split it into two components.
        let (digits, unit_char) = reading.split_at(reading.len() - 1);
        let number = digits.parse::<u32>()
            .map_err(|_| "invalid number in supply reading")?;
        // Get the first character and convert to lowercase. God rust likes to be verbose.
        let unit = match unit_char.as_bytes()[0].to_ascii_lowercase() as char {
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => return Err("missing level-suffix in supply reading"),
            'l' => 1,
            'm' => 2,
            'h' => 3,
            '?' => -1,
            _ => return Err("invalid unit in supply reading")?
        };

        return Ok((number as i32, unit));
    }

    // 1 or zero characters, just do a direct match.
    match reading {
        "?" => Ok((-1, -1)),
        "-" => Ok((0, 0)),
        "0" => Ok((0, 0)),
        ""  => Err("empty supply reading"),
        _   => Err("invalid supply reading"),
    }
}


/// Calculates the stellar-grid key for a pair of coordinates.
pub fn stellar_grid_key(x: f64, y: f64, z: f64) -> u64 {
  let gx = ((x / 32.0) as i32) & (8192-1) as i32;
  let gy = ((y / 32.0) as i32) & (8192-1) as i32;
  let gz = ((z / 32.0) as i32) & (8192-1) as i32;

  let key = ((gx as u64) << 26) | ((gy as u64) << 13) | (gz as u64);

  key
}


#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use super::*;

    #[test]
    fn test_count_file_lines_no_newlines() {
        // create a temp file with no content.
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.flush().unwrap();
        assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), 0);

        // write some content with no newlines
        write!(tmpfile, "no newline").unwrap();
        tmpfile.flush().unwrap();
        assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), 0);

        // now write every character except newline
        for i in 0..256 {
            if i != 10 {
                write!(tmpfile, "{}", i as u8).unwrap();
            }
        }
        tmpfile.flush().unwrap();
        assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), 0);
    }

    #[test]
    fn test_count_file_lines_just_newlines() {
        let mut tmpfile = NamedTempFile::new().unwrap();

        for i in 1..257 {
            tmpfile.write("\n".as_bytes()).unwrap();
            tmpfile.flush().unwrap();
            assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), i);
        }
    }

    #[test]
    fn test_count_file_lines_mixed() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        let mut buf: [u8; 65536] = [0; 65536];
        let mut lines: usize = 0;

        // Fill the buffer
        for i in 0..65536 {
            let start = i / 256;
            let count = i % 256;
            let end = start + count + 1;
            for c in i..end {
                buf[i] = c as u8;
                if c == 10 {
                    lines += 1
                }
            }
        }
        tmpfile.write(&buf).unwrap();
        tmpfile.flush().unwrap();

        assert_ne!(lines, 0);
        assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), lines);
    }

    #[test]
    fn test_parse_supply_level_invalid() {
        // form a string that starts with a digit and ends with a valid level suffix,
        // but has a non-numeric character that will cause the digits.parse to fail
        assert_eq!(parse_supply_level("0:?"), Err("invalid number in supply reading"));
        assert_eq!(parse_supply_level("0123123.m"), Err("invalid number in supply reading"));
        // pass a number too large for a uint32
        assert_eq!(parse_supply_level("9999999999999999999m"), Err("invalid number in supply reading"));

        // pass a value that doesn't end with a valid suffix.
        assert_eq!(parse_supply_level("00"), Err("missing level-suffix in supply reading"));

        // pass a value that doesn't start with a digit
        assert_eq!(parse_supply_level("?m"), Err("malformed supply reading"));

        // try some invalid single chars.
        assert_eq!(parse_supply_level("!"), Err("invalid supply reading"));
        assert_eq!(parse_supply_level("a"), Err("invalid supply reading"));
        assert_eq!(parse_supply_level("1"), Err("invalid supply reading"));

        // lastly, empty string.
        assert_eq!(parse_supply_level(""), Err("empty supply reading"));
    }

    #[test]
    fn test_parse_supply_level_unknown() {
        // "unknown" indicated by two -1s
        assert_eq!(parse_supply_level("?"), Ok((-1, -1)));
    }

    #[test]
    fn test_parse_supply_level_zero() {
        // "zero"
        assert_eq!(parse_supply_level("-"), Ok((0, 0)));
        assert_eq!(parse_supply_level("0"), Ok((0, 0)));
    }


    #[test]
    fn test_parse_supply_level_values() {
        // 0 units, unknown level
        assert_eq!(parse_supply_level("0?"), Ok((0, -1)));

        // 10 units, low
        assert_eq!(parse_supply_level("10l"), Ok((10, 1)));

        // 1000 units, low caps
        assert_eq!(parse_supply_level("1000L"), Ok((1000, 1)));

        // 424242 units
        assert_eq!(parse_supply_level("424242?"), Ok((424242, -1)));
        assert_eq!(parse_supply_level("424242l"), Ok((424242, 1)));
        assert_eq!(parse_supply_level("424242m"), Ok((424242, 2)));
        assert_eq!(parse_supply_level("424242h"), Ok((424242, 3)));

        // 2134567891 units
        assert_eq!(parse_supply_level("2134567891L"), Ok((2134567891, 1)));
        assert_eq!(parse_supply_level("2134567891M"), Ok((2134567891, 2)));
        assert_eq!(parse_supply_level("2134567891H"), Ok((2134567891, 3)));
    }

    #[test]
    fn test_stellar_grid_key_zero() {
        let actual_key = stellar_grid_key(0., 0., 0.);
        assert_eq!(0, actual_key);
    }

    #[test]
    fn test_stellar_grid_key_near_zero() {
        // where -32 < n < 32, we should come out to zero also
        assert_eq!(0, stellar_grid_key(1.0, 1.0, 1.0));
        assert_eq!(0, stellar_grid_key(-1.0, -1.0, -1.0));
        assert_eq!(0, stellar_grid_key(31.0, -31.0, 31.0));
        assert_eq!(0, stellar_grid_key(-31.0, 31.0, -31.0));
    }

    #[test]
    fn test_stellar_grid_key_non_ones() {
        assert_eq!(1, stellar_grid_key(0., 0., 32.));
        assert_eq!(1<<13, stellar_grid_key(0., 32., 0.));
        assert_eq!(1<<26, stellar_grid_key(32., 0., 0.));
        assert_eq!((1<<26)+(1<<13)+1, stellar_grid_key(32., 32., 32.));
    }

    #[test]
    fn test_stellar_grid_key_negatives() {
        // -1 should be represented as 8192-1
        let neg1 = (8192-1) as u64;
        assert_eq!((neg1<<26)+(neg1<<13)+neg1, stellar_grid_key(-32., -32., -32.));
    }

    #[test]
    fn test_stellar_grid_key() {
        // for the key: 42, -420, 69 we expect:
        //  gx = (42/32) -> 1 << 26
        //  gy = (-420/32) = -31 -> (8192-31) << 13
        //  gz = (69/32) = 2
        let actual = stellar_grid_key(42.1238, -420.03823, 69.3837);
        let gx = actual >> 26;
        let gy = actual >> 13 & (8192-1);
        let gz = actual & (8192-1);
        assert_eq!(gx, 42/32);
        assert_eq!(gy, 8192 - 420/32);
        assert_eq!(gz, 69 / 32);
    }
}
