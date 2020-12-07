pub const CSV_TYPE_FLAG: u32 = 1 << 22;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CsvTimedelta {
    Blockheight(u16),
    Seconds512(u16),
}

impl CsvTimedelta {
    pub fn sequence(self) -> u32 {
        match self {
            CsvTimedelta::Blockheight(block_height) => {
                block_height as u32
            }
            CsvTimedelta::Seconds512(seconds) => {
                (seconds as u32) | CSV_TYPE_FLAG
            }
        }
    }
}
