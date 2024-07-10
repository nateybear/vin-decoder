pub fn vin_descriptor(v: &str) -> String {
    let mut v = format!("{:*<17}", v);
    v.replace_range(8..9, "*");
    let descriptor = &v[..11];
    if &v[2..3] == "9" {
        v[..14].to_owned()
    } else {
        descriptor.to_uppercase()
    }
}
