use super::format::*;

pub struct Dump {
    pub path: String,
    pub step: u32,
    pub last_update: u32,
    pub ds0: DataSource,
    pub ds1: DataSource,
    pub rra: [RRA; 8],
}

impl Dump {
    pub fn new(path: &str) -> Self {
        // Genera ogni elemento richiamando una closure per gli indici da 0 a 7
        let rra_list: [RRA; 8] = std::array::from_fn(|_index| RRA::default());

        Dump {
            path: path.to_string(),
            step: 0,
            last_update: 0,
            ds0: DataSource::default(),
            ds1: DataSource::default(),
            rra: rra_list,
        }
    }

    pub fn from_xml(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Failed to read XML file");
        let dump: RawDump = quick_xml::de::from_str(&content).expect("Failed to parse XML");

        let mut rra_list: [RRA; 8] = std::array::from_fn(|_index| RRA::default());
        for (i, archive) in dump.archives.into_iter().enumerate() {
            if i < 8 {
                rra_list[i] = archive;
            }
        }

        Dump {
            path: path.to_string(),
            step: dump.step,
            last_update: dump.lastupdate as u32,
            ds0: dump.data_sources.get(0).cloned().unwrap_or_default(),
            ds1: dump.data_sources.get(1).cloned().unwrap_or_default(),
            rra: rra_list,
        }
    }
    
    pub fn to_xml(&self, path: &str) -> String {
        let raw_dump = RawDump {
            version: "0003".to_string(),
            step: self.step,
            lastupdate: self.last_update as u64,
            data_sources: vec![self.ds0.clone(), self.ds1.clone()],
            archives: self.rra.to_vec(),
        };

        let xml_content = quick_xml::se::to_string(&raw_dump).expect("Failed to serialize XML");

        if !path.is_empty() {
            std::fs::write(path, &xml_content).expect("Failed to write XML file");
        }

        xml_content
    }

    pub fn find_max(&self, rra_idx: u32) -> Option<RRDSample> {
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
   
    /// Rileva sequenze continue anomale in base al numero di deviazioni
    /// standard dalla media (z-score threshold).
    /// `ds_index`: 0 per ds0, 1 per ds1.
    /// `k_sigma`: solitamente compreso tra 2.5 e 3.5.
    pub fn find_spikes_by_zscore(&self, rra_idx: u32, ds_index: usize, k_sigma: f64) -> Vec<Spike> {
        if rra_idx >= self.rra.len() as u32 || ds_index > 1 {
            return Vec::new();
        }

        let rra = &self.rra[rra_idx as usize];
        
        // Estrai solo i valori validi (!NaN) per calcolare media e deviazione standard
        let valid_values: Vec<f64> = rra
            .database
            .rows
            .iter()
            .map(|row| row.values[ds_index])
            .filter(|v| !v.is_nan())
            .collect();

        if valid_values.is_empty() {
            return Vec::new();
        }

        let count = valid_values.len() as f64;
        let mean = valid_values.iter().sum::<f64>() / count;
        let variance = valid_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        // Se la deviazione standard è 0 non ci sono anomalie rilevabili
        if std_dev == 0.0 {
            return Vec::new();
        }

        let threshold = mean + k_sigma * std_dev;

        let threshold_ds0 = if ds_index == 0 { Some(threshold) } else { None };
        let threshold_ds1 = if ds_index == 1 { Some(threshold) } else { None };

        self.find_spikes_by_threshold(rra_idx, threshold_ds0, threshold_ds1)
    }

    /// Rileva sequenze di campioni consecutivi che superano le soglie
    /// specificate per ds0 o ds1.
    pub fn find_spikes_by_threshold(
        &self,
        rra_idx: u32,
        threshold_ds0: Option<f64>,
        threshold_ds1: Option<f64>,
    ) -> Vec<Spike> {
        if rra_idx >= self.rra.len() as u32 {
            return Vec::new();
        }

        let rra = &self.rra[rra_idx as usize];
        let mut spikes = Vec::new();
        let mut current_spike: Option<Spike> = None;

        for (idx, row) in rra.database.rows.iter().enumerate() {
            let v0 = row.values[0];
            let v1 = row.values[1];

            // Considera uno spike se supera la soglia specificata (e non è NaN)
            let is_spike = (!v0.is_nan() && threshold_ds0.map_or(false, |th| v0 >= th))
                || (!v1.is_nan() && threshold_ds1.map_or(false, |th| v1 >= th));

            if is_spike {
                let timestamp = self.timestamp_for_row_index(rra_idx, idx);
                let sample: RRDSample = (timestamp, v0, v1);

                if let Some(ref mut spike) = current_spike {
                    spike.end_index = idx;
                    spike.samples.push(sample);
                } else {
                    let prev_sample = if idx > 0 {
                        let prev_row = &rra.database.rows[idx - 1];
                        let prev_ts = self.timestamp_for_row_index(rra_idx, idx - 1);
                        Some((prev_ts, prev_row.values[0], prev_row.values[1]))
                    } else {
                        None
                    };
                    current_spike = Some(Spike {
                        start_index: idx,
                        end_index: idx,
                        prev_sample,
                        next_sample: None,
                        samples: vec![sample],
                    });
                }
            } else if let Some(mut spike) = current_spike.take() {
                let next_ts = self.timestamp_for_row_index(rra_idx, idx);
                spike.next_sample = Some((next_ts, v0, v1));
                spikes.push(spike);
            }
        }

        if let Some(spike) = current_spike {
            spikes.push(spike);
        }
        
        spikes
    }

    /// Rattoppa (sostituisce) i campioni di uno spike all'interno del database dell'RRA.
    /// Interpola linearmente tra `prev_sample` e `next_sample` aggiungendo opzionalmente
    /// un fattore di variazione pseudocasuale (es. `jitter_percent` = 0.05 per ±5%).
    pub fn patch_spike(&mut self, rra_idx: u32, spike: &Spike, jitter_percent: f64) {
        if rra_idx >= self.rra.len() as u32 || spike.samples.is_empty() {
            return;
        }

        // Determina i valori di riferimento iniziale e finale
        let (p0, p1) = match (spike.prev_sample, spike.next_sample) {
            (Some(prev), Some(next)) => ((prev.1, next.1), (prev.2, next.2)),
            (Some(prev), None) => ((prev.1, prev.1), (prev.2, prev.2)),
            (None, Some(next)) => ((next.1, next.1), (next.2, next.2)),
            (None, None) => return, // Nessun estremo disponibile per interpolare
        };

        let steps = (spike.samples.len() + 1) as f64;
        let rra = &mut self.rra[rra_idx as usize];

        for (i, row_idx) in (spike.start_index..=spike.end_index).enumerate() {
            let factor = (i + 1) as f64 / steps;

            // Interpolazione lineare base
            let mut v0 = p0.0 + (p0.1 - p0.0) * factor;
            let mut v1 = p1.0 + (p1.1 - p1.0) * factor;

            // Se jitter_percent > 0.0, aggiunge una lieve fluttuazione usando xorshift basato sul timestamp
            if jitter_percent > 0.0 {
                let ts = spike.samples[i].0;
                let rnd0 = pseudo_random_normalized(ts.wrapping_add(i as u64));
                let rnd1 = pseudo_random_normalized(ts.wrapping_add((i as u64) * 31 + 7));

                v0 *= 1.0 + (rnd0 * 2.0 - 1.0) * jitter_percent;
                v1 *= 1.0 + (rnd1 * 2.0 - 1.0) * jitter_percent;
            }

            rra.database.rows[row_idx].values = [v0, v1];
        }
    }

    /// Calcola il timestamp corrispondente all'indice di riga specificato.
    fn timestamp_for_row_index(&self, rra_idx: u32, index: usize) -> u64 {
        let rra = &self.rra[rra_idx as usize];
        let total_rows = rra.database.rows.len();
        let rra_step = (self.step as u64) * (rra.pdp_per_row as u64);
        let last_rra_time = self.last_update as u64 - (self.last_update as u64 % rra_step);
        let offset_from_end = (total_rows - 1 - index) as u64;
        last_rra_time - (offset_from_end * rra_step)
    }
}

/// Semplice generatore di numeri pseudocasuali uniformi in [0.0, 1.0)
/// senza dipendenze esterne (Xorshift64).
fn pseudo_random_normalized(seed: u64) -> f64 {
    let mut x = if seed == 0 { 0xdeadbeef_cafebabe } else { seed };
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x as f64) / (u64::MAX as f64)
}