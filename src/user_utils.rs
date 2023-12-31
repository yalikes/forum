use time::PrimitiveDateTime;

#[inline]
pub fn check_session_valid(assign_time: PrimitiveDateTime, life_time: time::Duration) -> bool {
    let current_time = time::OffsetDateTime::now_utc();
    let current_time = time::PrimitiveDateTime::new(current_time.date(), current_time.time());
    current_time < assign_time + life_time
}
