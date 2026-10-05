mod rrd;

use clap::{Parser, Subcommand};
use rrd::types::RRAType;
use crate::rrd::types::rra_idx_to_string;

const K_SIGMA : f64 = 3.0;

#[derive(Parser, Debug)]
#[command(
    name = "mrtgtool",
    version,
    about = "Analysis and elaboration tool for MRTG-generated RRD files"
)]

struct Cli {
    #[command(subcommand)]
    command: Commands,


}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Find the maximum value for a given RRA
    FindMax {
        /// Path of input XML file
        #[arg(short, long, default_value = "in.xml")]
        input: String,

        /// RRA to analyze
        #[arg(long, value_enum)]
        rra: RRAType,
    },

    /// Remove spikes from an RRD file
    DeSpike {
        /// Path of input XML file
        #[arg(short, long, default_value = "in.xml")]
        input: String,

        /// Path of output XML file
        #[arg(short, long, default_value = "out.xml")]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::FindMax {
            input,
            rra
        } => {
            let dump = rrd::Dump::from_xml(&input);
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
            input,
            output,
        } => {
            let mut dump = rrd::Dump::from_xml(&input);
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