use std::fmt;

use shared::entities::Chunk;

struct DebugChunk {
    chunk: Chunk,
}

impl DebugChunk {
    #[allow(dead_code)]
    fn new(chunk: Chunk) -> Self {
        DebugChunk { chunk }
    }
}

impl fmt::Display for DebugChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const COLS: i32 = 32;
        let mut out: String = "".to_string();
        let mut counter = 0;

        for i in self.chunk.get_block_storage().iter() {
            if counter == 0 {
                out += "\r\n";
            }

            out += &i.to_string();

            counter += 1;
            counter %= COLS;
        }

        write!(f, "{out}").unwrap();
        Ok(())
    }
}

fn main() {

}