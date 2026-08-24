use std::f32::consts::PI;

pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = 2.0 * PI;
    let wrapped = angle % two_pi;
    if wrapped < 0.0 {
        wrapped + two_pi
    } else {
        wrapped
    }
}

pub fn selected_sector(angle: f32, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let sector_angle = 2.0 * PI / count as f32;
    let shifted = angle + PI / 2.0 + sector_angle / 2.0;
    (normalize_angle(shifted) / sector_angle).floor() as usize % count
}
