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

    pub fn print_info(&self) {
        println!("Container path: {}", self.path);
        println!("Last update: {}", self.last_update);
    }
}