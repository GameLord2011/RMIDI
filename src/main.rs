use std::{
    fs,
    io::{Error, ErrorKind, stdin},
    path::Path,
};

/**
    The header track.
 */
struct HeaderTrack {
    flag: [u8; 4],
    length: u32,
    mode: u16,
    num_tracks: u16,
    tpq: u16,
}

/**
    A generic track.
 */
struct Track {
    flag: [u8; 4],
    length: u32,
    data: Vec<u8>,
}

/**
    A MIDI message. 
 */
#[derive(Clone)]
struct Message {
    delta: u32, // Erm acktually the MIDI format uses Variable Length Quantities to represent the delta.
    status: u8,
    data: Vec<u8>,
}

impl Message {
    /**
        Creates a new (empty) message and returns it.
     */
    fn new() -> Self {
        Self {
            delta: 0,
            status: 0,
            data: vec![]
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    let mut path = String::new();
    if args.len() > 1 {
        path = args[1].clone();
    } else {
        println!("Whar is the file:");
        stdin().read_line(&mut path).unwrap();
    }
    if let Some('\n') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\r') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\"') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\"') = path.chars().rev().next_back() {
        path.remove(0);
    }
    if let Some('\'') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\'') = path.chars().rev().next_back() {
        path.remove(0);
    }

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }
    let p = Path::new(&path);
    let bytes = fs::read(p).unwrap();

    let header: HeaderTrack = HeaderTrack {
        flag: [bytes[0], bytes[1], bytes[2], bytes[3]],
        length: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        mode: u16::from_be_bytes([bytes[8], bytes[9]]),
        num_tracks: u16::from_be_bytes([bytes[10], bytes[11]]),
        tpq: u16::from_be_bytes([bytes[12], bytes[13]]),
    };

    if header.flag != [0x4D, 0x54, 0x68, 0x64] {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "The header flag is incorrect.",
        ));
    }

    let mut tracks: Vec<Track> = vec![];

    let mut next_chunk_offset: usize = (header.length + 8).try_into().unwrap();
    for _ in 0..header.num_tracks {
        println!("Offset: {}", next_chunk_offset);
        let l = u32::from_be_bytes([
            bytes[next_chunk_offset + 4],
            bytes[next_chunk_offset + 5],
            bytes[next_chunk_offset + 6],
            bytes[next_chunk_offset + 7],
        ]);
        tracks.insert(
            tracks.len(),
            Track {
                flag: [
                    bytes[next_chunk_offset],
                    bytes[next_chunk_offset + 1],
                    bytes[next_chunk_offset + 2],
                    bytes[next_chunk_offset + 3],
                ],
                length: l,
                data: bytes[(next_chunk_offset + 8)..(next_chunk_offset + 8 + l as usize)].to_vec(),
            },
        );
        next_chunk_offset += 8 + l as usize;
    }

    let mut valid_tracks = true;
    let mut first_inv_chunk = 0;

    for (i, j) in tracks.iter().enumerate() {
        if (j.flag != [0x4D, 0x54, 0x72, 0x6B]) || !j.data.ends_with(&[0x00, 0xFF, 0x2F, 0x00]) {
            valid_tracks = false;
            first_inv_chunk = i;
            break;
        }
    }

    if valid_tracks == false {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Chunk {first_inv_chunk} has an invalid flag or no end command!"),
        ));
    }

    // let mut deltas: Vec<u8> = vec![];

    for track in tracks {
        let d = track.data;
        let mut offset: usize = 0;
        // let mut messages: Vec<Message> = vec![];
        loop {
            // let mut message = Message::new();
            println!("{}", offset);
            let possible_bytes = [d[offset], d[offset + 1], d[offset + 2], d[offset + 3]];
            let mut bytes: Vec<u8> = vec![];
            for byte in possible_bytes {
                if (byte & 0x80) == 0x80 {
                    bytes.insert(bytes.len(), byte);
                    continue;
                } else {
                    bytes.insert(bytes.len(), byte);
                    break;
                }
            }

            let mut vlq: u32 = 0;

            if bytes.len() == 4 {
                vlq = u32::from_be_bytes([
                    bytes[0],
                    bytes[1],
                    bytes[2],
                    bytes[3],
                ]);
            } else if bytes.len() == 3 {
                vlq = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
            } else if bytes.len() == 2 {
                vlq = u32::from_be_bytes([0, 0, bytes[0], bytes[1]]);
            } else if bytes.len() == 1 {
                vlq = u32::from_be_bytes([0, 0, 0, bytes[0]]);
            }

            let mut n = vlq & 0x7F;

            // More-or-less copy-pasted from ImHex's MIDI 1.0 pattern.
            if vlq & 0x8000 == 0x8000 {
                n += ((vlq & 0x7f00) >> 8) * 0x80;
            }
            if vlq & 0x800000 == 0x800000 {
                n += ((vlq & 0x7f0000) >> 8 * 2) * 0x4000;
            }
            if vlq & 0x80000000 == 0x80000000 {
                n += ((vlq & 0x7f000000) >> 8 * 3) * 0x200000;
            }

            println!("D = {}", n);

            let status = d[offset + bytes.len()];

            match status {
                0x80..=0x8F /* Note off */ => {
                    offset += bytes.len() + 3;
                },
                0x90..=0x9F /* Note on */ => {
                    offset += bytes.len() + 3;
                },
                0xFA /* Start */ => {println!("Note start")},
                0xFB /* Continue */ => {println!("Continue")},
                0xFC /* End */ => {println!("Note end")},
                0xFF /* Meta Event */ => {
                    println!("Meta event.");
                    match d[offset + bytes.len() + 1] {
                        0x01..=0x06 /* String-related things */ => {
                            println!("String event");
                            offset += d[offset + bytes.len() + 2] as usize + bytes.len() + 3;
                        },  
                        0x2F /* End of track */ => {
                            println!("Eot; {}, {:X}", status, d[offset + bytes.len() + 1]);
                            break;
                        },
                        _ => {
                            println!("Unknown meta event; type is {:X}; length is {} bytes", d[offset + bytes.len() + 1], d[offset + bytes.len() + 2]);
                            offset += d[offset + bytes.len() + 2] as usize + bytes.len() + 3;
                        }
                    }
                },
                _ => {println!("Unknown status {:X}", status)}
            }
        }
    }

    Ok(())
}
