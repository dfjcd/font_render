mod flag_bit;

use byteorder::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek},
};

use raylib::{misc::AsF32, prelude::*};

use crate::flag_bit::FlagBits;

fn main() -> Result<(), anyhow::Error> {
    // let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let mut file = load_file()?;
    let sfnt_version = file.read_u32::<BigEndian>()?;
    println!("sfnt_version: {sfnt_version}");

    let num_tables = file.read_u16::<BigEndian>()?;
    println!("Number of Tables: {}", num_tables);

    let search_range = file.read_u16::<BigEndian>()?;
    let entry_selector = file.read_u16::<BigEndian>()?;
    let range_shift = file.read_u16::<BigEndian>()?;
    println!("Search Range: {}", search_range);
    println!("Entry Selector: {}", entry_selector);
    println!("Range Shift: {}", range_shift);

    let mut tables = HashMap::<String, FontTable>::new();

    for _ in 0..num_tables {
        let tag = get_tag(&mut file)?;
        let checksum = file.read_u32::<BigEndian>()?;
        let offset = file.read_u32::<BigEndian>()?;
        let length = file.read_u32::<BigEndian>()?;

        tables.insert(tag.clone(), FontTable::new(&tag, checksum, offset, length));
    }

    file.rewind()?;
    let glyf = tables.get("glyf").unwrap();

    file.seek(std::io::SeekFrom::Start(glyf.offset as u64))?;

    let num_contures = file.read_i16::<BigEndian>()?;
    let is_simple = num_contures >= 0;
    let x_min = file.read_i16::<BigEndian>()?;
    let y_min = file.read_i16::<BigEndian>()?;
    let x_max = file.read_i16::<BigEndian>()?;
    let y_max = file.read_i16::<BigEndian>()?;
    let data = match is_simple {
        true => {
            let mut end_points_of_contures = Vec::<i16>::with_capacity(num_contures as usize);
            for i in 0..num_contures {
                end_points_of_contures.insert(i as usize, file.read_i16::<BigEndian>()?);
            }

            let instructions_length = file.read_u16::<BigEndian>()?;
            let mut flags = 0;
            let mut instructions = Vec::<u8>::new();
            if instructions_length == 0 {
                flags = file.read_u8()?;
            } 
            else {
                for i in 0..instructions_length {
                    instructions.insert(i as usize, file.read_u8()?);
                }
                flags = file.read_u8()?;
            }
            let x_coordinates = file.read_u16::<BigEndian>()?;
            let y_coordinates = file.read_u16::<BigEndian>()?;
            GlyfTableData::Simple {
                end_points_of_contures,
                instructions_length,
                instructions,
                flags,
                x_coordinates,
                y_coordinates,
            }
        }
        false => GlyfTableData::Complex {},
    };

    let glyf = GlyfTable {
        num_contures,
        x_min,
        y_min,
        x_max,
        y_max,
        data,
    };

    println!("{glyf:?}");

    // for i in 0..8 {
    //     match glyf.data {
    //         GlyfTableData::Simple {
    //             end_points_of_contures,
    //             instructions_length,
    //             instructions,
    //             flags,
    //             x_coordinates,
    //             y_coordinates,
    //         } => {
    //             let is_active = FlagBits::is_bit_active(flags, i);
    //             println!("bit: {i} is active: {is_active}");
    //         }
    //         GlyfTableData::Complex {} => todo!(),
    //     }
    // }

    // while !rl.window_should_close() {
    //     let mut d = rl.begin_drawing(&thread);
    //     d.clear_background(Color::WHITE);
    // }

    Ok(())
}

fn load_file() -> Result<File, anyhow::Error> {
    let file = File::open("")?;
    Ok(file)
}

fn get_tag(file: &mut File) -> Result<String, anyhow::Error> {
    let buf = [0u8; 4];
    let mut tag = String::new();

    for _ in 0..buf.len() {
        let b = file.read_u8()? as char;
        tag.push(b);
    }

    Ok(tag)
}

#[derive(Debug)]
struct FontTable {
    tag: String,
    checksum: u32,
    offset: u32,
    length: u32,
}

impl FontTable {
    fn new(tag: &str, checksum: u32, offset: u32, length: u32) -> Self {
        FontTable {
            tag: tag.to_string(),
            checksum,
            offset,
            length,
        }
    }
}

#[derive(Debug)]
struct GlyfTable {
    num_contures: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    data: GlyfTableData,
}

#[derive(Debug)]
enum GlyfTableData {
    Simple {
        end_points_of_contures: Vec<i16>,
        instructions_length: u16,
        instructions: Vec::<u8>,
        flags: u8,
        x_coordinates: u16,
        y_coordinates: u16,
    },
    Complex {},
}
