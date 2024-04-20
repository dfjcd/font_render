#![allow(dead_code, unused_variables, unused_assignments)]

mod flag_bit;
mod config;

use byteorder::*;
use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    io::Seek,
};

use crate::{config::Config, flag_bit::FlagBits};

fn main() -> Result<(), anyhow::Error> {
    // let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    dotenv::dotenv().ok();
    let config = Config::parse();

    let mut file = load_file(&config.font_path)?;
    let sfnt_version = file.read_u32::<BigEndian>()?;
    println!("sfnt_version: {sfnt_version}");

    let num_tables = file.read_u16::<BigEndian>()?;
    println!("Number of Tables: {}", num_tables);

    let _search_range = file.read_u16::<BigEndian>()?;
    let _entry_selector = file.read_u16::<BigEndian>()?;
    let _range_shift = file.read_u16::<BigEndian>()?;

    let mut tables = HashMap::<String, FontTable>::new();

    for _ in 0..num_tables {
        let tag = get_tag(&mut file)?;
        let checksum = file.read_u32::<BigEndian>()?;
        let offset = file.read_u32::<BigEndian>()?;
        let length = file.read_u32::<BigEndian>()?;

        tables.insert(tag.clone(), FontTable::new(&tag, checksum, offset, length));
    }

    file.rewind()?;

    let maxp = tables.get("maxp").unwrap();

    file.seek(std::io::SeekFrom::Start(maxp.offset as u64))?;

    let major = file.read_u16::<BigEndian>()?;
    let minor = file.read_u16::<BigEndian>()?;
    let maxp = MaxProfile {
        version: (minor / 10) as f32 + major as f32,
        num_glyphs: file.read_u16::<BigEndian>()?,
    };

    println!("maxp: {maxp:#?}");

    let glyf = tables.get("glyf").unwrap();
    file.rewind()?;

    file.seek(std::io::SeekFrom::Start(glyf.offset as u64))?;

    for _ in 0..maxp.num_glyphs {
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
                } else {
                    for i in 0..instructions_length {
                        instructions.insert(i as usize, file.read_u8()?);
                    }
                    flags = file.read_u8()?;
                }
                let mut flags_count = 1;

                if FlagBits::is_bit_active(flags, 3) {
                    flags_count = file.read_u8()?;
                }

                let x_coordinates: u16 = if FlagBits::is_bit_active(flags, 1) {
                    file.read_u8()? as u16
                } else {
                    file.read_u16::<BigEndian>()?
                };

                let y_coordinates: u16 = if FlagBits::is_bit_active(flags, 2) {
                    file.read_u8()? as u16
                } else {
                    file.read_u16::<BigEndian>()?
                };

                let simple = GlyfSimple {
                    end_points_of_contures,
                    instructions_length,
                    instructions,
                    flags,
                    flags_count,
                    x_coordinates,
                    y_coordinates,
                };

                GlyfTableData::Simple(simple)
            }
            false => GlyfTableData::Composite(GlyfComposite {
                argument1: 0,
                argument2: 0,
                flags: 0,
                glyph_index: 0,
            }),
        };
        let glyf = GlyfTable {
            num_contures,
            x_min,
            y_min,
            x_max,
            y_max,
            data,
        };
        match &glyf.data {
            GlyfTableData::Simple(simple_glyf) => {
                let on_curve = FlagBits::is_bit_active(simple_glyf.flags, 0);
                let is_x_short = FlagBits::is_bit_active(simple_glyf.flags, 1);
                let is_y_short = FlagBits::is_bit_active(simple_glyf.flags, 2);
                let repeat = FlagBits::is_bit_active(simple_glyf.flags, 3);
                let x_is_same_or_sign = FlagBits::is_bit_active(simple_glyf.flags, 4);
                let y_is_same_or_sign = FlagBits::is_bit_active(simple_glyf.flags, 5);
                let reserved_1 = FlagBits::is_bit_active(simple_glyf.flags, 6);
                let reserved_2 = FlagBits::is_bit_active(simple_glyf.flags, 7);
                let flags_count = simple_glyf.flags_count;
                println!("flags: on_curve: {on_curve} | is_x_short: {is_x_short} | is_y_short: {is_y_short} | repeat: {repeat} | flags_counts: {flags_count} | x: {x_is_same_or_sign} | y: {y_is_same_or_sign} | reserved1: {reserved_1} | reserved2: {reserved_2}");
            }
            GlyfTableData::Composite(_) => todo!(),
        }
        println!("contures: {num_contures}, x_min: {x_min}, y_min: {y_min}, x_max: {x_max}, y_max: {y_max}");
    }

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

fn load_file(path: &str) -> Result<File, anyhow::Error> {
    let file = File::open(path)?;
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
    Simple(GlyfSimple),
    Composite(GlyfComposite),
}

#[derive(Debug)]
struct GlyfSimple {
    end_points_of_contures: Vec<i16>,
    instructions_length: u16,
    instructions: Vec<u8>,
    flags: u8,
    flags_count: u8,
    x_coordinates: u16,
    y_coordinates: u16,
}

#[derive(Debug)]
struct GlyfComposite {
    flags: u16,
    glyph_index: u16,
    argument1: u16,
    argument2: u16,
}

#[derive(Debug)]
struct MaxProfile {
    version: f32,
    num_glyphs: u16,
}