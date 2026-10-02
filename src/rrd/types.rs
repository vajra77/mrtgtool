use clap::ValueEnum;

pub type RRDFloat = f64;
pub type RRDUlong = u64;

pub type RRDSample = (RRDUlong, RRDFloat, RRDFloat);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum RRAType {
    DailyAvg,
    WeeklyAvg,
    MonthlyAvg,
    YearlyAvg,
    DailyMax,
    WeeklyMax,
    MonthlyMax,
    YearlyMax,
}

impl RRAType {
    pub fn to_index(self) -> u32 {
        match self {
            RRAType::DailyAvg => 0,
            RRAType::WeeklyAvg => 1,
            RRAType::MonthlyAvg => 2,
            RRAType::YearlyAvg => 3,
            RRAType::DailyMax => 4,
            RRAType::WeeklyMax => 5,
            RRAType::MonthlyMax => 6,
            RRAType::YearlyMax => 7,
        }
    }
}