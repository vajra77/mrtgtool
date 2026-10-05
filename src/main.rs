mod rrd;

use clap::{Parser, Subcommand, ValueEnum};
use rrd::types::RRAType;
use crate::rrd::types::rra_idx_to_string;

const K_SIGMA : f64 = 3.0;

#[derive(Parser, Debug)]
#[command(name = "mrtgtool", version, about = "Strumento di analisi ed elaborazione dump RRD MRTG")]
struct Cli {
    /// Percorso del file XML da analizzare
    #[arg(short, long, default_value = "assets/dati.xml", global = true)]
    input: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Trova il valore massimo per una specifica RRA
    FindMax {
        /// RRA da analizzare
        #[arg(long, value_enum)]
        rra: RRAType,
    },
    /// Rileva e gestisce gli spike di traffico
    DeSpike {
        #[arg(long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut dump = rrd::Dump::from_xml(&cli.input);

    match cli.command {
        Commands::FindMax { rra } => {
            let rra_idx = rra.to_index();
            match dump.find_max(rra_idx) {
                Some((ts, max_in, max_out)) => {
                    println!("Max value for {:?}: ({:e}, {:e}) at {}", rra, max_in, max_out, ts);
                }
                None => {
                    println!("No data found for {:?}", rra);
                }
            }
        }
        Commands::DeSpike {
            output,
        } => {
            for rra_idx in 0 ..= 8u32 {
                let spikes_in = dump.find_spikes_by_zscore(rra_idx, 0, K_SIGMA);
                println!("Found {} input spikes in {:?}", spikes_in.len(), rra_idx_to_string(rra_idx));
                for (i, spike) in spikes_in.iter().enumerate() {
                    println!(
                        "  Spike #{}: from row {} to row {} (samples: {})",
                        i + 1,
                        spike.start_index,
                        spike.end_index,
                        spike.samples.len()
                    );
                    println!("... patching ...");
                    dump.patch_spike(rra_idx, spike, 0.05);
                }

                let spikes_out = dump.find_spikes_by_zscore(rra_idx, 1, K_SIGMA);
                println!("Found {} input spikes out {:?}", spikes_out.len(), rra_idx_to_string(rra_idx));
                for (i, spike) in spikes_out.iter().enumerate() {
                    println!(
                        "  Spike #{}: from row {} to row {} (samples: {})",
                        i + 1,
                        spike.start_index,
                        spike.end_index,
                        spike.samples.len()
                    );
                    println!("... patching ...");
                    dump.patch_spike(rra_idx, spike, 0.05);
                }
            }
            println!("Dumping to XML file");
            dump.to_xml(&output);
        }
    }
}