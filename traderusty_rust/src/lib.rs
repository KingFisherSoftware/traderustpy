use std::fs::File;
use std::io::{self, BufReader, Read};
use bytecount::count as byte_counter;

/// Counts the number of '\n's in a file as quickly as possible and then
/// returns the count.
#[allow(dead_code)]
fn count_file_lines(filename: &str) -> io::Result<usize> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; 65536];    // 64kb at a time
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
        let mut buf: [u8; 512] = [0; 512];
        for i in 0..512 {
            buf[i] = (i % 256) as u8;
        }
        let mut lines: usize = 0;
        for i in 0..256 {
            for j in 1..257 {
                let end = i+j;
                tmpfile.write(&buf[i..end]).unwrap();
                if i <= 10 && end > 10 {
                    lines += 1;
                }
                if i <= 266 && end > 266 {
                    lines += 1;
                }
            }
        }
        tmpfile.flush().unwrap();
        assert_ne!(lines, 0);
        assert_eq!(count_file_lines(tmpfile.path().to_str().unwrap()).unwrap(), lines);
    }
}
