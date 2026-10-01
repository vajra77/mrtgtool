use crate::rrd::types::RRDUlong;
use super::format::*;

pub struct Container {
    pub path: String,
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
        let mut idx = 0;
        let mut max_idx = 0;
        let nsamples = rra.database.rows.len() as u32;

        for row in iter {
            if row.values[0] > max_sample.values[0] || row.values[0] > max_sample.values[1] {
                max_sample = row;
                max_idx = nsamples - idx;
            }
            idx += 1;
        }

        let last_sample = match rra_idx {
            0 | 4 => self.last_update - (self.last_update % 300) + 600,
            1 | 5 => self.last_update - (self.last_update % 1800) + 3600,
            2 | 6 => self.last_update - (self.last_update % 7200) + 14400,
            3 | 7 => self.last_update - (self.last_update % 86400) + 172800,
            _ => 0,
        };

        let timestamp = match rra_idx {
            0 | 4 => last_sample - (300 * max_idx),
            1 | 5 => last_sample - (1800 * max_idx),
            2 | 6 => last_sample - (7200 * max_idx),
            3 | 7 => last_sample - (86400 * max_idx),
            _ => 0,
        };
        let result : RRDSample = (timestamp as RRDUlong, max_sample.values[0], max_sample.values[1]);
        Some(result)
    }

    pub fn print_info(&self) {
        println!("Container path: {}", self.path);
        println!("Last update: {}", self.last_update);
    }
}