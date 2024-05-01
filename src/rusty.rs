use bytecount::count as byte_counter;
use std::fs::File;
use std::io::{self, BufReader, Read};

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
        let number = digits
            .parse::<u32>()
            .map_err(|_| "invalid number in supply reading")?;
        // Get the first character and convert to lowercase. God rust likes to be verbose.
        let unit = match unit_char.as_bytes()[0].to_ascii_lowercase() as char {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                return Err("missing level-suffix in supply reading")
            }
            'l' => 1,
            'm' => 2,
            'h' => 3,
            '?' => -1,
            _ => return Err("invalid unit in supply reading")?,
        };

        return Ok((number as i32, unit));
    }

    // 1 or zero characters, just do a direct match.
    match reading {
        "?" => Ok((-1, -1)),
        "-" => Ok((0, 0)),
        "0" => Ok((0, 0)),
        "" => Err("empty supply reading"),
        _ => Err("invalid supply reading"),
    }
}

/// Calculates the stellar-grid key for a pair of coordinates.
/*
ATOW current populated values for x, y, and z are:
│ min(pos_x)  │ max(pos_x) │ min(pos_y) │ max(pos_y) │ min(pos_z) │ max(pos_z)  │
│ -42213.8125 │ 40503.8125 │ -3381.375  │ 5319.21875 │ -16899.75  │ 65630.15625 │

dividing by 32 proved to produce a good balance between bucket count/bucket size,
│  min x /32  │ max x /32  │ min y / 32 │ max y / 32 │ min z / 32 │ max z / 32  │
│ -1319.18164 │ 1265.74414 │ -105.66796 │ 166.225585 │ -528.11718 │ 2050.942382 │

-> x (-1320,+1266), y (-106, +167), z (-529, +2051)
next power of two:
->   (-2048,+2048),   (-128, +256),   (-1024, +4096)

I think it's reasonable to imagine those limits being exceeded at least once, so
we can assume that we need to store values of upto +/- 8192 for each coordinate,
which calls for signed 16-bit compartments.
 */
fn stellar_grid_key_component(component: f64) -> i16 {
    (component / 32.).floor() as i16
}

pub fn stellar_grid_key(x: f64, y: f64, z: f64) -> u64 {
    // I've chosen to make 'y' the most-significant word here because it currently
    // has the least range since the galaxy is disk-like, and because it represents
    // galactic "north/south".
    // Promote gy into a u32 so that negatives fill all the most significant bits:
    //  0xffffi16 -> i64 -> u64 = 0xffffffffffffffff
    // where i16 -> u16 -> u64 = 0x000000000000ffff
    let gy = stellar_grid_key_component(y) as i64 as u64;
    let gx = stellar_grid_key_component(x) as u16 as u64;
    let gz = stellar_grid_key_component(z) as u16 as u64;

    (gy << 32) | (gx << 16) | gz
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_count_file_lines_no_newlines() {
        // create a temp file with no content.
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.flush().unwrap();
        assert_eq!(
            count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(),
            0
        );

        // write some content with no newlines
        write!(tmpfile, "no newline").unwrap();
        tmpfile.flush().unwrap();
        assert_eq!(
            count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(),
            0
        );

        // now write every character except newline
        for i in 0..256 {
            if i != 10 {
                write!(tmpfile, "{}", i as u8).unwrap();
            }
        }
        tmpfile.flush().unwrap();
        assert_eq!(
            count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(),
            0
        );
    }

    #[test]
    fn test_count_file_lines_just_newlines() {
        let mut tmpfile = NamedTempFile::new().unwrap();

        for i in 1..257 {
            tmpfile.write("\n".as_bytes()).unwrap();
            tmpfile.flush().unwrap();
            assert_eq!(
                count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(),
                i
            );
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
        assert_eq!(
            count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(),
            lines
        );
    }

    #[test]
    fn test_parse_supply_level_invalid() {
        // form a string that starts with a digit and ends with a valid level suffix,
        // but has a non-numeric character that will cause the digits.parse to fail
        assert_eq!(
            parse_supply_level("0:?"),
            Err("invalid number in supply reading")
        );
        assert_eq!(
            parse_supply_level("0123123.m"),
            Err("invalid number in supply reading")
        );
        // pass a number too large for a uint32
        assert_eq!(
            parse_supply_level("9999999999999999999m"),
            Err("invalid number in supply reading")
        );

        // pass a value that doesn't end with a valid suffix.
        assert_eq!(
            parse_supply_level("00"),
            Err("missing level-suffix in supply reading")
        );

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
    fn test_sellar_grid_key_component() {
        // positive values should populate the space 0+,
        // while negative values should start at -1.
        assert_eq!(0i16, stellar_grid_key_component(0.));
        assert_eq!(0i16, stellar_grid_key_component(-0.));
        assert_eq!(0i16, stellar_grid_key_component(1.));
        assert_eq!(0i16, stellar_grid_key_component(2.));
        assert_eq!(0i16, stellar_grid_key_component(16.));
        assert_eq!(0i16, stellar_grid_key_component(31.9999));
        assert_eq!(1i16, stellar_grid_key_component(32.0));
        assert_eq!(1i16, stellar_grid_key_component(63.9999999999));
        assert_eq!(2i16, stellar_grid_key_component(64.));

        assert_eq!(-1i16, stellar_grid_key_component(-0.00001));
        assert_eq!(-1i16, stellar_grid_key_component(-1.));
        assert_eq!(-1i16, stellar_grid_key_component(-2.));
        assert_eq!(-1i16, stellar_grid_key_component(-16.));
        assert_eq!(-1i16, stellar_grid_key_component(-31.9999));
        assert_eq!(-1i16, stellar_grid_key_component(-32.0));
        assert_eq!(-2i16, stellar_grid_key_component(-32.000000001));
        assert_eq!(-2i16, stellar_grid_key_component(-63.9999999999));
        assert_eq!(-2i16, stellar_grid_key_component(-64.));
        assert_eq!(-3i16, stellar_grid_key_component(-64.0000001));
    }

    #[test]
    fn test_stellar_grid_key_zero() {
        let actual_key = stellar_grid_key(0., 0., 0.);
        assert_eq!(0, actual_key);
    }

    #[test]
    fn test_stellar_grd_key_minus_one() {
        let actual_key = stellar_grid_key(-1., -1., -1.);
        assert_eq!(-1i64 as u64, actual_key);
    }

    #[test]
    fn test_stellar_grid_key_near_zero() {
        // where -32 < n < 32, we should come out to zero also
        assert_eq!(0, stellar_grid_key(1.0, 1.0, 1.0));
        assert_eq!(0, stellar_grid_key(31.0, 31.0, 31.0));

        assert_ne!(0, stellar_grid_key(0.,  0.,  32.));
        assert_ne!(0, stellar_grid_key(0.,  32.,  0.));
        assert_ne!(0, stellar_grid_key(0.,  32.,  32.));
        assert_ne!(0, stellar_grid_key(32.,  0.,  0.));
        assert_ne!(0, stellar_grid_key(32.,  0.,  32.));
        assert_ne!(0, stellar_grid_key(32.,  32.,  0.));
        assert_ne!(0, stellar_grid_key(32.,  32.,  32.));
    }

    #[test]
    fn test_stellar_grid_key_near_minus_1() {
        let neg1key = -1i64 as u64;

        assert_eq!(neg1key, stellar_grid_key(-1.0, -1.0, -1.0));
        assert_eq!(neg1key, stellar_grid_key(-32.0, -32.0, -32.0));

        assert_ne!(neg1key, stellar_grid_key( -0.,   -0.,   -32.1));
        assert_ne!(neg1key, stellar_grid_key( -0.,   -32.1, -0.));
        assert_ne!(neg1key, stellar_grid_key( -0.,   -32.1, -321.));
        assert_ne!(neg1key, stellar_grid_key(-32.1,  -0.,   -0.));
        assert_ne!(neg1key, stellar_grid_key(-32.1,  -0.,   -321.));
        assert_ne!(neg1key, stellar_grid_key(-32.1,  -32.1, -0.));
        assert_ne!(neg1key, stellar_grid_key(-32.1,  -32.1, -32.1));
    }

    #[test]
    fn test_stellar_grid_key_ordering_pos() {
        // 1.0, 2.0, 3.0 should be -> 0x00 0x02 0x01 0x03
        let result = stellar_grid_key(32.0, 64.0, 96.0);
        assert_eq!(1, (result >> 16) & 0xff);
        assert_eq!(2, (result >> 32) & 0xffff);
        assert_eq!(3, (result >> 0 ) & 0xff);
    }

    #[test]
    fn test_stellar_grid_key_ordering_neg() {
        let result = stellar_grid_key(-33.0, -65.0, -97.0);
        // this should be: (-3 << 32) | (-2 << 16) | (-4)
        // aka: 0xfffffffefffffffd
        let expectation = 0xfffffffdfffefffc;
        assert_eq!(expectation, result);
    }
}
