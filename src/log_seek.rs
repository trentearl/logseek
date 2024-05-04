use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
};

use chrono::{DateTime, Utc};

use crate::date_utils::date_from_line;

pub struct SeekableItem {
    pub sort: DateTime<Utc>,
    pub value: String,
    pub pos: u64,
}

pub struct Seekable {
    reader: BufReader<File>,
    pub next: Option<SeekableItem>,
    pub last: SeekableItem,
}

impl Seekable {
    pub fn new(
        mut reader: BufReader<File>,
        len: u64,
        start: Option<DateTime<Utc>>,
    ) -> Option<Seekable> {
        let line = match read_last_line(&mut reader) {
            Some(line) => line.trim().to_string(),
            None => return None,
        };

        let last_date = match date_from_line(line.trim()) {
            Some(date) => date,
            None => return None,
        };
        let last_pos = len - line.len() as u64;
        let last = SeekableItem {
            sort: last_date,
            value: line,
            pos: last_pos,
        };

        match new_next(reader, len, start, last_date) {
            Some((reader, next)) => Some(Seekable {
                reader,
                next: Some(next),
                last,
            }),
            None => None,
        }
    }

    pub fn advance(&mut self) -> bool {
        match self.next {
            None => false,
            Some(ref item) => {
                let seekable = read_at(&mut self.reader, item.pos);
                self.next = seekable;
                true
            }
        }
    }
}

fn new_next(
    mut reader: BufReader<File>,
    len: u64,
    start: Option<DateTime<Utc>>,
    last_date: DateTime<Utc>,
) -> Option<(BufReader<File>, SeekableItem)> {
    let next = match read_at(&mut reader, 0) {
        Some(item) => item,
        None => return None,
    };

    if let Some(start) = start {
        if start > last_date {
            return None;
        } else if start >= next.sort {
            // We need to find the first line that is greater than or equal to the start date
            // binary search
            let mut low = 0;
            let mut high = len;
            let mut last_mid = high;
            while low < high {
                let mid = low + (high - low) / 2;

                let item = match read_at(&mut reader, mid) {
                    Some(item) => item,
                    None => return None,
                };
                if item.sort < start {
                    low = item.pos;
                } else {
                    high = item.pos
                }

                if mid == last_mid {
                    break;
                }
                last_mid = mid;
            }

            let mut item = match read_at(&mut reader, low) {
                Some(item) => item,
                None => return None,
            };

            while item.sort < start {
                let next = match read_at(&mut reader, item.pos) {
                    Some(next) => next,
                    None => return None,
                };
                item = next;
            }

            return Some((reader, item));
        }
    }

    return Some((reader, next));
}

fn read_at<R: BufRead + Seek>(reader: &mut R, pos: u64) -> Option<SeekableItem> {
    match reader.seek(SeekFrom::Start(pos)) {
        Ok(_) => (),
        Err(_) => return None,
    };

    let mut line = String::new();
    let size = match reader.read_line(&mut line) {
        Ok(size) => size as u64,
        Err(_) => return None,
    };
    let new_pos = size + pos;

    let value = line.trim();

    if let Some(date) = date_from_line(value) {
        return Some(SeekableItem {
            sort: date,
            value: value.to_string(),
            pos: new_pos,
        });
    }
    match reader.seek(SeekFrom::Start(new_pos)) {
        Ok(_) => (),
        Err(_) => return None,
    };

    let mut line2 = String::new();
    let size2 = match reader.read_line(&mut line2) {
        Ok(size) => size as u64,
        Err(_) => return None,
    };

    let new_pos2 = size2 + new_pos;

    let value2 = line2.trim();

    match date_from_line(value2) {
        Some(date) => Some(SeekableItem {
            sort: date,
            value: value2.to_string(),
            pos: new_pos2,
        }),
        None => None,
    }
}

fn read_last_line<R: BufRead + Seek>(file: &mut R) -> Option<String> {
    let mut len = match file.seek(SeekFrom::End(0)) {
        Ok(len) => len,
        Err(_) => return None,
    };

    if len == 0 {
        return None;
    }

    let mut last_char = 0 as u8;
    let mut seen_line = false;

    while len > 0 {
        len -= 1;
        match file.seek(SeekFrom::Start(len)) {
            Ok(_) => (),
            Err(_) => return None,
        }
        match file.read_exact(std::slice::from_mut(&mut last_char)) {
            Ok(_) => (),
            Err(_) => return None,
        }

        if seen_line && last_char == b'\n' {
            break;
        } else {
            seen_line = true;
        }
    }

    let mut last_line = String::new();
    match file.seek(SeekFrom::Start(len + 1)) {
        Ok(_) => (),
        Err(_) => return None,
    }

    match file.read_line(&mut last_line) {
        Ok(_) => (),
        Err(_) => return None,
    }

    Some(last_line.trim_end().to_string())
}

impl PartialEq for SeekableItem {
    fn eq(&self, other: &Self) -> bool {
        self.sort == other.sort
    }
}

impl Eq for SeekableItem {}

impl PartialOrd for SeekableItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for SeekableItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.sort.cmp(&self.sort)
    }
}

impl PartialEq for Seekable {
    fn eq(&self, other: &Self) -> bool {
        self.next.eq(&other.next)
    }
}

impl Eq for Seekable {}

impl PartialOrd for Seekable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Seekable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.next {
            Some(ref next) => match other.next {
                Some(ref other_next) => next.cmp(other_next),
                None => std::cmp::Ordering::Greater,
            },
            None => match other.next {
                Some(_) => std::cmp::Ordering::Less,
                None => std::cmp::Ordering::Equal,
            },
        }
    }
}
