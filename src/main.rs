mod rrd;

use rrd::types::*;

fn main() {
    let c1 = rrd::Container::parse_xml("assets/dati.xml");
    let max = c1.find_max_in_rra(MONTHLY_AVG_IDX);
    match max {
        Some((timestamp, max_in, max_out)) => {
            println!("Max value: ({:.2},{:.2}) at timestamp {}", max_in, max_out, timestamp);
        }
        None => {
            println!("No data found");
        }
    }
}