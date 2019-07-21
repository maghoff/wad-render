pub fn render(buf: &mut [u8; 320 * 200]) {
    for px in buf.iter_mut() {
        *px = 100;
    }
}
