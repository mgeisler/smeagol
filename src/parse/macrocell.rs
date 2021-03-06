/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Macrocell Life patterns.
//!
//! TODO: clean up this module and actually use it.
use nom::{line_ending, not_line_ending};
use std::io::Read;

named!(whitespace, take_while!(|c: u8| (c as char).is_whitespace()));

named!(comment_line<&[u8], &[u8]>,
    do_parse!(
        char!('#') >>
        comment: not_line_ending >>
        opt!(line_ending) >>
        (comment)
    )
);

named!(header<&[u8], ()>,
    do_parse!(
        tag!("[M2]") >>
        not_line_ending >>
        line_ending >>
        ()
    )
);

named!(level_3<&[u8], Cell>,
    do_parse!(
        cells: many0!(one_of!(".*$")) >>
        opt!(not_line_ending) >>
        opt!(line_ending) >>
        (Cell::LevelThree { cells })
    )
);

named!(interior<&[u8], Cell>,
    do_parse!(
        level: map_res!(nom::digit0, btoi::btoi) >>
        whitespace >>
        nw: map_res!(nom::digit0, btoi::btoi) >>
        whitespace >>
        ne: map_res!(nom::digit0, btoi::btoi) >>
        whitespace >>
        sw: map_res!(nom::digit0, btoi::btoi) >>
        whitespace >>
        se: map_res!(nom::digit0, btoi::btoi) >>
        opt!(not_line_ending) >>
        opt!(line_ending) >>
        (Cell::Interior { level, children: [nw, ne, sw, se] })
    )
);

fn macrocell(input: &[u8]) -> nom::IResult<&[u8], Vec<Cell>> {
    let (rest, _) = header(input)?;
    let (mut rest, _) = comment_line(rest)?;
    let mut cells = vec![];
    loop {
        if let Ok((new_rest, cell)) = interior(rest) {
            cells.push(cell);
            rest = new_rest;
        } else if let Ok((new_rest, cell)) = level_3(rest) {
            cells.push(cell);
            rest = new_rest;
        } else {
            break;
        }
    }
    Ok((rest, cells))
}

/// An individual cell in a Macrocell file.
#[derive(Clone, Debug)]
pub enum Cell {
    /// A level three cell.
    LevelThree { cells: Vec<char> },
    /// An interior cell.
    Interior { level: u8, children: [usize; 4] },
}

/// A Macrocell Life pattern.
pub struct Macrocell {
    pub cells: Vec<Cell>,
}

impl Macrocell {
    /// Loads a Macrocell pattern from the given file.
    pub fn from_file<P>(path: P) -> Result<Self, failure::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;

        let cells = macrocell(&buf).unwrap();

        Ok(Self { cells: cells.1 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file() {
        Macrocell::from_file("./assets/waterbear.mc").unwrap();
    }
}
