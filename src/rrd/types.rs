pub type RRDFloat = f64;
pub type RRDUlong = u64;

pub type RRDSample = (RRDFloat, RRDFloat);

pub enum RRAIndex {
    DailyAvg = 0,
    WeeklyAvg = 1,
    MonthlyAvg = 2,
    YearlyAvg = 3,
    DailyMax = 4,
    WeeklyMax = 5,
    MonthlyMax = 6,
    YearlyMax = 7,
}