use super::format::*;

pub struct Container {
    pub path: String,
    pub step: u32,
    pub last_update: u32,
    pub ds0: DataSource,
    pub ds1: DataSource,
    pub rra: [RRA; 8],
}

impl Container {
    pub fn new(path: &str) -> Self {
        // Genera ogni elemento richiamando una closure per gli indici da 0 a 7
        let rra_list: [RRA; 8] = std::array::from_fn(|_index| RRA::default());

        Container {
            path: path.to_string(),
            step: 0,
            last_update: 0,
            ds0: DataSource::default(),
            ds1: DataSource::default(),
            rra: rra_list,
        }
    }

    pub fn parse_xml(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Failed to read XML file");
        let dump: RRDDump = quick_xml::de::from_str(&content).expect("Failed to parse XML");

        let mut rra_list: [RRA; 8] = std::array::from_fn(|_index| RRA::default());
        for (i, archive) in dump.archives.into_iter().enumerate() {
            if i < 8 {
                rra_list[i] = archive;
            }
        }

        Container {
            path: path.to_string(),
            step: dump.step,
            last_update: dump.lastupdate as u32,
            ds0: dump.data_sources.get(0).cloned().unwrap_or_default(),
            ds1: dump.data_sources.get(1).cloned().unwrap_or_default(),
            rra: rra_list,
        }
    }

    pub fn find_max_in_rra(&self, rra_idx: u32) -> Option<RRDSample> {
        if rra_idx >= self.rra.len() as u32 {
            return None;
        }

        let rra = &self.rra[rra_idx as usize];

        let mut iter = rra.database.rows.iter();
        let mut max_sample = iter.next()?; // Restituisce None immediatamente se rows è vuoto
        let mut max_idx = 0;

        for (idx, row) in iter.enumerate() {
            let actual_idx = idx + 1;
            if row.values[0] > max_sample.values[0] || row.values[1] > max_sample.values[1] {
                max_sample = row;
                max_idx = actual_idx;
            }
        }
        let timestamp = self.timestamp_for_row_index(rra_idx, max_idx);
        let result : RRDSample = (timestamp, max_sample.values[0], max_sample.values[1]);
        Some(result)
    }

    pub fn print_info(&self) {
        println!("Container path: {}", self.path);
        println!("Last update: {}", self.last_update);
    }

    fn timestamp_for_row_index(&self, rra_idx: u32, index: usize) -> u64 {
        let rra = &self.rra[rra_idx as usize];
        let total_rows = rra.database.rows.len();
        let rra_step = (self.step as u64) * (rra.pdp_per_row as u64);
        let last_rra_time = self.last_update as u64 - (self.last_update as u64 % rra_step);
        let offset_from_end = (total_rows - 1 - index) as u64;
        last_rra_time - (offset_from_end * rra_step)
    }
}