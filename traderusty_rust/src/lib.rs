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
    use std::cmp::min;
    use std::io::Write;
    use super::*;
    fn spread_newlines(filename: &str, intervals: Vec<usize>) {
        let mut buffer = File::create(filename).unwrap();
        let mut bytes = Vec::with_capacity(255);
        for i in 1..256 {
            if i != 10 {
                bytes.push(i as u8);
            }
        }
        println!("capacity size = {}, first_byte = {}, last byte = {}", bytes.len(), bytes.first().unwrap(), bytes.last().unwrap());
        for interval in intervals.iter() {
            let mut remaining = *interval;
            while remaining > 0 {
                let count = min(remaining, bytes.len() - 1);
                buffer.write_all(&bytes[0..count]).unwrap();
                remaining -= count;
            }
            buffer.write("\n".as_bytes()).unwrap();
        }
    }

    #[test]
    fn test_count_file_lines() {
        spread_newlines("test1.txt", vec![0]);
        spread_newlines("test2.txt", vec![1]);
        spread_newlines("test3.txt", vec![0, 0]);
        spread_newlines("test4.txt", vec![0, 1, 2, 3, 4]);
        spread_newlines("test5.txt", vec![100, 1000, 10000]);
    }

}