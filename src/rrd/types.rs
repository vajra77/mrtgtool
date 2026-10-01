pub type RRDFloat = f64;
pub type RRDUlong = u64;

pub type RRDSample = (RRDUlong, RRDFloat, RRDFloat);

pub const DAILY_AVG_IDX: u32 = 0;
pub const WEEKLY_AVG_IDX: u32 = 1;
pub const MONTHLY_AVG_IDX: u32 = 2;
pub const YEARLY_AVG_IDX: u32 = 3;
pub const DAILY_MAX_IDX: u32 = 4;
pub const WEEKLY_MAX_IDX: u32 = 5;
pub const MONTHLY_MAX_IDX: u32 = 6;
pub const YEARLY_MAX_IDX: u32 = 7;