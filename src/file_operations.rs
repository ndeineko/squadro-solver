use std::fs::File;
use std::io::{self, Read, Write};

// Paths to data files.
pub const WINNING_STATES_PATH: [&str; 2] = ["player_0_wins.data", "player_1_wins.data"];
pub const ALL_STATES_PATH: &str = "all_states.data";

const CHUNK_SIZE_BYTES: usize = 1024 * 1024;
const CHUNK_SIZE_BITS: u64 = CHUNK_SIZE_BYTES as u64 * 8;

/// Return the value of bit `state_id` from the ZIP-compressed chunked bit-set stored in file `path`
pub fn read_state_value(path: &str, state_id: u64) -> bool {
    let file = File::open(path)
        .unwrap_or_else(|_| panic!("Unable to open file in read-only mode : {}", path));

    let mut zip_reader = zip::ZipArchive::new(file)
        .unwrap_or_else(|_| panic!("Unable to parse ZIP file : {}", path));

    let chunk_id: u64 = state_id / CHUNK_SIZE_BITS;
    let bit_index: u64 = state_id % CHUNK_SIZE_BITS;
    let byte_index: u64 = bit_index / 8;

    // Look for the chunk `chunk_id` in zip file.
    let mut chunk_file = match zip_reader.by_name(&format!("chunk{chunk_id}")) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => {
            // The chunk is absent when it's only made of 0s.
            return false;
        }
        Err(_) => panic!(
            "Unable to look for chunk {} in ZIP file : {}",
            chunk_id, path
        ),
    };

    if byte_index >= chunk_file.size() {
        // `byte_index` is part of (removed) 0s at the end of the chunk.
        return false;
    }

    if byte_index > 0 {
        // Drop the first `byte_index` bytes from the chunk.
        io::copy(&mut chunk_file.by_ref().take(byte_index), &mut io::sink()).unwrap_or_else(|_| {
            panic!(
                "Unable to skip the first {} bytes from chunk {} in ZIP file : {}",
                byte_index, chunk_id, path
            )
        });
    }

    // Read the value of the byte `byte_index` from the chunk.
    let mut buffer = [0u8];
    chunk_file.read_exact(&mut buffer).unwrap_or_else(|_| {
        panic!(
            "Unable to read byte {} from chunk {} in ZIP file : {}",
            byte_index, chunk_id, path
        )
    });

    // Return the value of the bit `bit_index` from the chunk.
    (buffer[0] >> (bit_index % 8)) & 1 == 1
}

/// Store `states` in a ZIP-compressed chunked bit-set file `path`
pub fn write_states(path: &str, states: &roaring::RoaringTreemap) {
    // Create a new file and open it in r+w mode.
    let file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap_or_else(|_| panic!("Unable to create file : {}", path));

    // Create an empty ZIP file.
    zip::ZipWriter::new(&file)
        .finish()
        .unwrap_or_else(|_| panic!("Unable to create an empty ZIP file : {}", path));

    let add_chunk = |chunk_buffer: &[u8], chunk_id: u64| {
        let mut zip_appender = zip::ZipWriter::new_append(&file)
            .unwrap_or_else(|_| panic!("Unable to parse ZIP file (in append mode) : {}", path));

        // Add a chunk (new file) to the ZIP file.
        zip_appender
            .start_file(
                format!("chunk{chunk_id}"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap_or_else(|_| {
                panic!("Unable to create chunk {} in ZIP file : {}", chunk_id, path)
            });

        // Add chunk contents.
        zip_appender
            .write_all(chunk_buffer)
            .unwrap_or_else(|_| panic!("Unable to add chunk {} to ZIP file : {}", chunk_id, path));

        // Write changes to ZIP file.
        zip_appender.finish().unwrap_or_else(|_| {
            panic!(
                "Unable to finalize writing chunk {} in ZIP file : {}",
                chunk_id, path
            )
        });
    };

    let mut chunk_buffer: Vec<u8> = Vec::with_capacity(CHUNK_SIZE_BYTES);
    let mut chunk_id: u64 = states.min().unwrap_or(0) / CHUNK_SIZE_BITS;

    for state_id in states.iter() {
        // Write `chunk_buffer` before it grows larger than `CHUNK_SIZE_BYTES`.
        if state_id / CHUNK_SIZE_BITS > chunk_id {
            add_chunk(&chunk_buffer, chunk_id);
            chunk_buffer = Vec::with_capacity(CHUNK_SIZE_BYTES);
            chunk_id = state_id / CHUNK_SIZE_BITS;
        }

        let bit_index: u64 = state_id % CHUNK_SIZE_BITS;
        let byte_index: usize = (bit_index / 8) as usize;

        if byte_index >= chunk_buffer.len() {
            // Grow `chunk_buffer` when necessary.
            chunk_buffer.resize(byte_index + 1, 0);
        }

        // Write 1 to bit `bit_index`.
        chunk_buffer[byte_index] |= 1 << (bit_index % 8);
    }

    if !chunk_buffer.is_empty() {
        add_chunk(&chunk_buffer, chunk_id);
    }
}

/// Terminate thread if `path` is an existing path in the file system
pub fn abort_if_path_exists(path: &str) {
    if std::path::Path::new(path).exists() {
        panic!("The following path already exists : {}\nThe process will be stopped now to avoid losing data!", path);
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::{Mutex, OnceLock, PoisonError};

    use super::*;

    pub fn run_in_tempdir<F>(f: F)
    where
        F: FnOnce(),
    {
        // Current dir is per-process (not per-thread), hence the lock.
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_current_dir(&tmp).unwrap();

        f();

        std::env::set_current_dir("..").unwrap();

        // Close and delete dir now, otherwise deletion may fail silently on drop.
        tmp.close().unwrap();
    }

    #[test]
    fn state_from_zip() {
        run_in_tempdir(|| {
            let file = File::options()
                .write(true)
                .create_new(true)
                .open("f")
                .unwrap();

            let mut zip = zip::ZipWriter::new(&file);
            zip.start_file("chunk17", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&[0b10000000, 0b00000000, 0b00000000, 0b00000001])
                .unwrap();
            zip.start_file("chunk0", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&[0b00000000, 0b00000000, 0b11111111, 0b00001000])
                .unwrap();
            zip.finish().unwrap();

            let at_max_100_bits = std::cmp::min(100, CHUNK_SIZE_BITS);
            for chunk_id in 0..=25 {
                let chunk_start_bit = chunk_id * CHUNK_SIZE_BITS;
                let chunk_end_bit = (chunk_id + 1) * CHUNK_SIZE_BITS;
                for i in (chunk_start_bit..=chunk_start_bit + at_max_100_bits)
                    .chain(chunk_end_bit - at_max_100_bits..chunk_end_bit)
                {
                    assert!(
                        read_state_value("f", i) == (i == 17 * CHUNK_SIZE_BITS + 7)
                            || (i == 17 * CHUNK_SIZE_BITS + 24)
                            || (i == 27)
                            || (16..24).contains(&i)
                    );
                }
            }
        });
    }

    #[test]
    fn states_to_zip() {
        let name_regex = regex::Regex::new("^chunk([1-9][0-9]*|0)$").unwrap();

        let mut states = {
            let mut marked_ids = [
                3,
                14,
                1592653589793238462u64,
                33 * CHUNK_SIZE_BITS + 8,
                327 * CHUNK_SIZE_BITS - 95,
            ];
            marked_ids.sort();
            roaring::RoaringTreemap::from_sorted_iter(marked_ids).unwrap()
        };

        run_in_tempdir(|| {
            write_states("states", &states);

            let mut zip = zip::ZipArchive::new(File::open("states").unwrap()).unwrap();
            for i in 0..zip.len() {
                let mut file = zip.by_index(i).unwrap();
                let file_name = file.name();
                if name_regex.is_match(file_name) {
                    let chunk_id = (file_name[5..]).parse::<u64>().unwrap();
                    let mut chunk_data = Vec::new();
                    file.read_to_end(&mut chunk_data).unwrap();
                    assert_eq!(file.size(), chunk_data.len().try_into().unwrap());

                    for chunk_bit_index in 0..file.size() * 8 {
                        let chunk_byte_index = chunk_bit_index / 8;
                        let bit_data =
                            (chunk_data[chunk_byte_index as usize] >> (chunk_bit_index % 8)) & 1;
                        if bit_data == 1 {
                            assert!(states.remove(CHUNK_SIZE_BITS * chunk_id + chunk_bit_index));
                        }
                    }
                }
            }
        });

        assert!(states.is_empty());
    }

    #[test]
    fn states_empty_to_zip() {
        run_in_tempdir(|| {
            write_states("states", &roaring::RoaringTreemap::new());

            let zip = zip::ZipArchive::new(File::open("states").unwrap()).unwrap();

            assert!(zip.is_empty());
            assert!(!read_state_value("states", 0));
            assert!(!read_state_value("states", 1));
            assert!(!read_state_value("states", u64::MAX));
        });
    }

    #[test]
    fn states_unique_to_zip() {
        run_in_tempdir(|| {
            write_states(
                "states",
                &roaring::RoaringTreemap::from_sorted_iter([u64::MAX]).unwrap(),
            );

            let zip = zip::ZipArchive::new(File::open("states").unwrap()).unwrap();

            assert_eq!(zip.len(), 1);
            assert!(!read_state_value("states", 0));
            assert!(!read_state_value("states", 1));
            assert!(!read_state_value("states", u64::MAX - 1));
            assert!(read_state_value("states", u64::MAX));
        });
    }

    #[test]
    fn mistake_protection() {
        run_in_tempdir(|| {
            File::create("exists.txt").unwrap();

            let result = std::panic::catch_unwind(|| {
                abort_if_path_exists("exists.txt");
            });
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .downcast::<String>()
                .unwrap()
                .contains("exists.txt"));

            let result = std::panic::catch_unwind(|| {
                abort_if_path_exists("absent.txt");
            });
            assert!(result.is_ok());
        });
    }
}
