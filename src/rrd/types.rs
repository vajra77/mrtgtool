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

pub fn rra_idx_to_string(idx : u32) -> &'static str {
    match idx {
        0 => "daily_avg",
        1 => "weekly_avg",
        2 => "monthly_avg",
        3 => "yearly_avg",
        4 => "daily_max",
        5 => "weekly_max",
        6 => "monthly_max",
        7 => "yearly_max",
        _ => "unknown",
    }
}