use std::{
    fs,
    io::{ErrorKind, stdin},
    path::Path,
};

struct HeaderChunk {
    flag: [u8; 4],
    length: u32,
    mode: u16,
    num_tracks: u16,
    tpq: u16
}

struct Chunk {
    flag: [u8; 4],
    length: u32,
    data: Vec<u8>
}

fn main() -> std::io::Result<()> {
    println!("Whar is the file:");
    let mut path = String::new();
    stdin().read_line(&mut path).unwrap();
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
    } else {
        let p = Path::new(&path);
        let bytes = fs::read(p).unwrap();

        let header: HeaderChunk = HeaderChunk {
            flag: [bytes[0], bytes[1], bytes[2], bytes[3]],
            length: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            mode: u16::from_be_bytes([bytes[8], bytes[9]]),
            num_tracks: u16::from_be_bytes([bytes[10], bytes[11]]),
            tpq: u16::from_be_bytes([bytes[12], bytes[13]])
        };

        if header.flag != [0x4D, 0x54, 0x68, 0x64]
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "The header flag is incorrect.",
            ));
        }

        let mut tracks: Vec<Chunk> = vec![];

        let mut next_chunk_offset: usize = (header.length + 8).try_into().unwrap();
        for _ in 0..header.num_tracks {
            println!("Offset: {}", next_chunk_offset);
            let l = u32::from_be_bytes([bytes[next_chunk_offset + 4], bytes[next_chunk_offset + 5], bytes[next_chunk_offset + 6], bytes[next_chunk_offset + 7]]);
            tracks.insert(tracks.len(), Chunk {
                flag: [bytes[next_chunk_offset], bytes[next_chunk_offset + 1], bytes[next_chunk_offset + 2], bytes[next_chunk_offset + 3]],
                length: l,
                data: bytes[(next_chunk_offset + 4)..(next_chunk_offset + 4 + l as usize)].to_vec()
            });
            next_chunk_offset += 8 + l as usize;
        }
    }

    Ok(())
}
