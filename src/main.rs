mod rrd;

fn main() {
    let c1 = rrd::Container::parse_xml("assets/dati.xml");
    c1.print_info();
}