use time::PrimitiveDateTime;

#[inline]
pub fn now() -> PrimitiveDateTime {
    let current_time = time::OffsetDateTime::now_utc();
    PrimitiveDateTime::new(current_time.date(), current_time.time())
}
