mod rrd;

use rrd::types::*;

fn main() {
    let c1 = rrd::Container::parse_xml("assets/dati.xml");
    let max = c1.find_max_in_rra(MONTHLY_AVG_IDX);
    match max {
        Some((ts, max_in, max_out)) => {
            println!("Max value: ({:e},{:e}) at {}", max_in, max_out, ts);
        }
        None => {
            println!("No data found");
        }
    }
}