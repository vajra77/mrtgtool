mod rrd;

use clap::{Parser, Subcommand, ValueEnum};
use rrd::types::*;

#[derive(Parser, Debug)]
#[command(name = "mrtgtool", version, about = "Strumento di analisi ed elaborazione dump RRD MRTG")]
struct Cli {
    /// Percorso del file XML da analizzare
    #[arg(short, long, default_value = "assets/dati.xml", global = true)]
    file: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Trova il valore massimo per una specifica RRA
    FindMax {
        /// RRA da analizzare
        #[arg(long, value_enum)]
        rra: RraType,
    },
    /// Rileva e gestisce gli spike di traffico
    DeSpike {
        /// Algoritmo di rilevamento spike
        #[arg(long, value_enum)]
        alg: DeSpikeAlg,

        /// RRA su cui cercare gli spike
        #[arg(long, value_enum, default_value = "daily-avg")]
        rra: RraType,

        /// Canale dati: 0 (ds0) o 1 (ds1)
        #[arg(long, default_value_t = 0)]
        ds: usize,

        /// Moltiplicatore k-sigma (usato se alg = zScore)
        #[arg(long, default_value_t = 3.0)]
        k_sigma: f64,

        /// Soglia valore assoluto per ds0 (usata se alg = thresh)
        #[arg(long)]
        thresh_ds0: Option<f64>,

        /// Soglia valore assoluto per ds1 (usata se alg = thresh)
        #[arg(long)]
        thresh_ds1: Option<f64>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum RraType {
    DailyAvg,
    WeeklyAvg,
    MonthlyAvg,
    YearlyAvg,
    DailyMax,
    WeeklyMax,
    MonthlyMax,
    YearlyMax,
}

impl RraType {
    fn to_index(self) -> u32 {
        match self {
            RraType::DailyAvg => DAILY_AVG_IDX,
            RraType::WeeklyAvg => WEEKLY_AVG_IDX,
            RraType::MonthlyAvg => MONTHLY_AVG_IDX,
            RraType::YearlyAvg => YEARLY_AVG_IDX,
            RraType::DailyMax => DAILY_MAX_IDX,
            RraType::WeeklyMax => WEEKLY_MAX_IDX,
            RraType::MonthlyMax => MONTHLY_MAX_IDX,
            RraType::YearlyMax => YEARLY_MAX_IDX,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum DeSpikeAlg {
    #[value(name = "zScore")]
    ZScore,
    #[value(name = "thresh")]
    Thresh,
}

fn main() {
    let cli = Cli::parse();
    let container = rrd::Container::parse_xml(&cli.file);

    match cli.command {
        Commands::FindMax { rra } => {
            let rra_idx = rra.to_index();
            match container.find_max(rra_idx) {
                Some((ts, max_in, max_out)) => {
                    println!("Max value for {:?}: ({:e}, {:e}) at {}", rra, max_in, max_out, ts);
                }
                None => {
                    println!("No data found for {:?}", rra);
                }
            }
        }
        Commands::DeSpike {
            alg,
            rra,
            ds,
            k_sigma,
            thresh_ds0,
            thresh_ds1,
        } => {
            let rra_idx = rra.to_index();
            let spikes = match alg {
                DeSpikeAlg::ZScore => container.find_spikes_by_zscore(rra_idx, ds, k_sigma),
                DeSpikeAlg::Thresh => container.find_spikes_by_threshold(rra_idx, thresh_ds0, thresh_ds1),
            };

            println!("Trovati {} spike con algoritmo {:?} su {:?}", spikes.len(), alg, rra);
            for (i, spike) in spikes.iter().enumerate() {
                println!(
                    "  Spike #{}: da riga {} a {} (campioni: {})",
                    i + 1,
                    spike.start_index,
                    spike.end_index,
                    spike.samples.len()
                );
            }
        }
    }
}